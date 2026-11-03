//! macOS 原生通知（UNUserNotificationCenter 直连）。
//!
//! tauri-plugin-notification 桌面端没有点击回调，点通知无法唤起面板；
//! 这里自建 delegate 接管点击事件：用户点横幅 → 主线程唤起面板。
//! `init` 必须在应用完成启动前于主线程调用（delegate 装晚了点击回调不生效）。

use std::sync::{Mutex, OnceLock};

use block2::{DynBlock, RcBlock};
use objc2::define_class;
use objc2::rc::Retained;
use objc2::runtime::{NSObject, NSObjectProtocol, ProtocolObject};
use objc2::{msg_send, ClassType};
use objc2_foundation::{NSError, NSString};
use objc2_user_notifications::{
    UNAuthorizationOptions, UNMutableNotificationContent, UNNotificationDefaultActionIdentifier,
    UNNotificationRequest, UNNotificationResponse, UNUserNotificationCenter,
    UNUserNotificationCenterDelegate,
};
use tauri::AppHandle;

/// delegate 回调要用的 AppHandle（delegate 无状态，走全局）
static PANEL_APP: OnceLock<Mutex<Option<AppHandle>>> = OnceLock::new();

/// Send/Sync 包装：delegate 对象无 ivars，回调由系统队列串行派发，仅用于 setDelegate 保活
struct SendDelegate(Retained<ProtocolObject<dyn UNUserNotificationCenterDelegate>>);
unsafe impl Send for SendDelegate {}
unsafe impl Sync for SendDelegate {}
static DELEGATE: OnceLock<SendDelegate> = OnceLock::new();

define_class!(
    #[unsafe(super(NSObject))]
    struct GlintNotifDelegate;

    unsafe impl NSObjectProtocol for GlintNotifDelegate {}

    unsafe impl UNUserNotificationCenterDelegate for GlintNotifDelegate {
        // 用户与通知横幅交互（点击 = DefaultAction）时系统回调
        #[unsafe(method(userNotificationCenter:didReceiveNotificationResponse:withCompletionHandler:))]
        unsafe fn did_receive_response(
            &self,
            _center: &UNUserNotificationCenter,
            response: &UNNotificationResponse,
            handler: &DynBlock<dyn Fn()>,
        ) {
            let tapped = response.actionIdentifier().to_string()
                == UNNotificationDefaultActionIdentifier.to_string();
            if tapped {
                if let Some(app) = PANEL_APP.get().and_then(|m| m.lock().unwrap().clone()) {
                    // 窗口操作必须回主线程
                    let runner = app.clone();
                    let _ = runner.run_on_main_thread(move || crate::show_panel(&app, false));
                }
            }
            // 必须调用 completion handler 通知系统处理完毕
            handler.call(());
        }
    }
);

impl GlintNotifDelegate {
    fn new() -> Retained<Self> {
        unsafe { msg_send![GlintNotifDelegate::class(), new] }
    }
}

/// 启动时调用（主线程）：装 delegate + 请求通知授权（已授权时幂等）
pub fn init(app: AppHandle) {
    let _ = PANEL_APP.set(Mutex::new(Some(app)));
    let center = UNUserNotificationCenter::currentNotificationCenter();

    let _ = DELEGATE.set(SendDelegate(ProtocolObject::from_retained(GlintNotifDelegate::new())));
    if let Some(d) = DELEGATE.get() {
        center.setDelegate(Some(&d.0));
    }

    let handler = RcBlock::new(|_granted: objc2::runtime::Bool, _err: *mut NSError| {});
    center.requestAuthorizationWithOptions_completionHandler(
        UNAuthorizationOptions::Alert | UNAuthorizationOptions::Sound,
        &handler,
    );
}

/// 发一条本地通知（任意线程；内部切主线程）。`id` 用仓库名，便于未来按仓库定向交互。
pub fn notify(app: &AppHandle, title: &str, body: &str, id: &str) {
    let title = title.to_string();
    let body = body.to_string();
    let id = id.to_string();
    let _ = app.run_on_main_thread(move || {
        let center = UNUserNotificationCenter::currentNotificationCenter();
        let content = UNMutableNotificationContent::new();
        content.setTitle(&NSString::from_str(&title));
        content.setBody(&NSString::from_str(&body));
        let request = UNNotificationRequest::requestWithIdentifier_content_trigger(
            &NSString::from_str(&id),
            &content,
            None,
        );
        let _ = center.addNotificationRequest_withCompletionHandler(&request, None);
    });
}

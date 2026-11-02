import AppKit
import UserNotifications

final class Notifier: NSObject, UNUserNotificationCenterDelegate {
    static let shared = Notifier()

    private let center: UNUserNotificationCenter? = {
        // 脱离 .app 包直接 swift run 时没有 bundle id，UNUserNotificationCenter 会崩溃，降级为仅日志。
        guard Bundle.main.bundleIdentifier != nil else { return nil }
        return UNUserNotificationCenter.current()
    }()

    func requestAuth() {
        guard let center else { return }
        center.delegate = self
        center.requestAuthorization(options: [.alert, .sound]) { _, _ in }
    }

    func notify(title: String, body: String, url: String?) {
        guard let center else { return }
        let content = UNMutableNotificationContent()
        content.title = title
        content.body = body
        content.sound = .default
        if let url { content.userInfo["url"] = url }
        center.add(UNNotificationRequest(identifier: UUID().uuidString, content: content, trigger: nil)) { _ in }
    }

    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification,
        withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void
    ) {
        completionHandler([.banner, .sound])
    }

    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        didReceive response: UNNotificationResponse,
        withCompletionHandler completionHandler: @escaping () -> Void
    ) {
        if let url = response.notification.request.content.userInfo["url"] as? String,
           let target = URL(string: url) {
            NSWorkspace.shared.open(target)
        }
        completionHandler()
    }
}

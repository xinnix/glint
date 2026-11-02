import AppKit
import SwiftUI

@main
struct GitCIStatusApp: App {
    @StateObject private var app = AppState.shared
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var delegate

    var body: some Scene {
        MenuBarExtra {
            PanelView()
                .environmentObject(app)
        } label: {
            Image(nsImage: app.menubarImage)
        }
        .menuBarExtraStyle(.window)
    }
}

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(.accessory)
        AppState.shared.start()
    }
}

final class WindowManager {
    static let shared = WindowManager()
    private var managerWindow: NSWindow?

    func openManager(app: AppState) {
        if let window = managerWindow, window.isVisible {
            window.makeKeyAndOrderFront(nil)
            NSApp.activate(ignoringOtherApps: true)
            return
        }
        let hosting = NSHostingView(rootView: ManagerView().environmentObject(app))
        let height = max(480, min(720, hosting.fittingSize.height))
        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 460, height: height),
            styleMask: [.titled, .closable],
            backing: .buffered,
            defer: false
        )
        window.title = "GitCIStatus · 仓库管理"
        window.contentView = hosting
        window.isReleasedWhenClosed = false
        window.center()
        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
        managerWindow = window
    }
}

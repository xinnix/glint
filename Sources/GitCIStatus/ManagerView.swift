import SwiftUI

struct ManagerView: View {
    @EnvironmentObject var app: AppState
    @State private var newRepo = ""
    @State private var tokenInput = ""
    @State private var message: String?

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 14) {
                section("监控仓库") {
                    VStack(spacing: 6) {
                        ForEach(app.config.repos, id: \.self) { repo in
                            HStack {
                                Text(repo)
                                Spacer()
                                Button {
                                    app.removeRepo(repo)
                                } label: {
                                    Image(systemName: "minus.circle")
                                        .foregroundStyle(.red)
                                }
                                .buttonStyle(.borderless)
                            }
                        }
                        if app.config.repos.isEmpty {
                            Text("暂无仓库")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    }
                    HStack {
                        TextField("owner/repo，支持批量粘贴（空格/逗号/换行分隔）", text: $newRepo)
                            .textFieldStyle(.roundedBorder)
                        Button("添加") {
                            let count = app.addRepos(newRepo)
                            message = count > 0
                                ? "已添加 \(count) 个仓库"
                                : "没有新增（每项格式应为 owner/repo）"
                            newRepo = ""
                        }
                        .disabled(newRepo.trimmingCharacters(in: .whitespaces).isEmpty)
                    }
                    HStack(spacing: 10) {
                        Button("导入 gh 账号的仓库") {
                            let count = app.importRepos(ProjectScanner.scanGhRepos())
                            message = count > 0
                                ? "已导入 \(count) 个仓库"
                                : "没有导入（需要 gh auth login）"
                        }
                        Button("扫描本地 ~/code 目录") {
                            let count = app.importFromCodeDir()
                            message = count > 0 ? "已导入 \(count) 个仓库" : "没有发现新仓库"
                        }
                    }
                }

                section("刷新与通知") {
                    Picker("刷新间隔", selection: $app.config.intervalSeconds) {
                        Text("15 秒").tag(15)
                        Text("30 秒").tag(30)
                        Text("1 分钟").tag(60)
                        Text("2 分钟").tag(120)
                        Text("5 分钟").tag(300)
                    }
                    Toggle("失败时发系统通知", isOn: $app.config.notifyOnFailure)
                    Toggle("运行中转为通过时发通知", isOn: $app.config.notifyOnSuccess)
                }

                section("GitHub Token") {
                    Text("当前：\(app.tokenInfo)")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    HStack {
                        SecureField("粘贴 Personal Access Token（repo 权限即可）", text: $tokenInput)
                            .textFieldStyle(.roundedBorder)
                        Button("保存到钥匙串") {
                            do {
                                try TokenProvider.saveToKeychain(tokenInput)
                                app.invalidateToken()
                                message = "Token 已保存到钥匙串"
                                tokenInput = ""
                            } catch {
                                message = "保存失败：\(error.localizedDescription)"
                            }
                        }
                        .disabled(tokenInput.trimmingCharacters(in: .whitespaces).isEmpty)
                    }
                    Text("私有仓库或更高限流需要 Token。若已用 gh 登录（gh auth login），会自动复用，无需手动配置。")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                if let message {
                    Text(message)
                        .font(.caption)
                        .foregroundStyle(.orange)
                }

                Text("配置文件：\(ConfigStore.fileURL.path)")
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }
            .padding(16)
        }
        .frame(minWidth: 460, maxWidth: 460, minHeight: 400, maxHeight: 720)
        .onChange(of: app.config) { _ in
            app.saveConfig()
        }
    }

    @ViewBuilder
    private func section<Content: View>(_ title: String, @ViewBuilder content: () -> Content) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title).font(.headline)
            content()
        }
    }
}

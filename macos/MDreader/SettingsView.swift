import SwiftUI

struct SettingsView: View {
    @EnvironmentObject private var settings: SettingsStore

    var body: some View {
        Form {
            Section("外部编辑器") {
                TextField("应用名称",
                          text: $settings.settings.editorCommand,
                          prompt: Text("例如：Typora、Visual Studio Code"))
                Text("用 macOS「打开方式」语义调用该应用打开当前文件（等价于 open -a <应用名> <文件>）。")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Section("默认主题") {
                Picker("默认主题", selection: $settings.settings.themePref) {
                    Text("跟随系统").tag(ThemePref.system)
                    Text("浅色").tag(ThemePref.light)
                    Text("深色").tag(ThemePref.dark)
                }
                .pickerStyle(.radioGroup)
                .labelsHidden()
                Text("未单独切换主题的文档使用此设置；已单独切换的文档保留各自选择。")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
        .formStyle(.grouped)
        .frame(width: 460, height: 260)
    }
}

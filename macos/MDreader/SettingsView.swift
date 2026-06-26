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
        }
        .formStyle(.grouped)
        .frame(width: 460, height: 160)
    }
}

import SwiftUI

struct SettingsView: View {
    @EnvironmentObject private var settings: SettingsStore

    var body: some View {
        Form {
            Section("外部编辑器") {
                TextField("编辑器命令",
                          text: $settings.settings.editorCommand,
                          prompt: Text("例如：code  或  open -a 'Visual Studio Code'"))
                Text("命令会在 shell（/bin/sh -c）中执行，当前文件路径会以引号包裹追加在末尾。")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
        .formStyle(.grouped)
        .frame(width: 460, height: 160)
    }
}

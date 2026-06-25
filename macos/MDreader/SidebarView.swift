import SwiftUI

enum SidebarTab: Hashable {
    case library, outline
}

struct SidebarView: View {
    @EnvironmentObject private var model: ReaderModel
    @State private var tab: SidebarTab = .library

    var body: some View {
        VStack(spacing: 0) {
            Picker("侧栏", selection: $tab) {
                Text("库").tag(SidebarTab.library)
                Text("大纲").tag(SidebarTab.outline)
            }
            .pickerStyle(.segmented)
            .padding(8)

            switch tab {
            case .library:
                LibraryView()
            case .outline:
                OutlineView()
            }
        }
        .frame(minWidth: 240)
    }
}

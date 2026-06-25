import SwiftUI

struct OutlineView: View {
    @EnvironmentObject private var model: ReaderModel

    var body: some View {
        if model.outline.isEmpty {
            VStack(spacing: 6) {
                Spacer()
                Image(systemName: "list.bullet.indent")
                    .font(.system(size: 28))
                    .foregroundStyle(.secondary)
                Text("无大纲").foregroundStyle(.secondary)
                Text("打开带标题的文档后显示")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Spacer()
            }
            .frame(maxWidth: .infinity)
        } else {
            ScrollView {
                VStack(alignment: .leading, spacing: 1) {
                    ForEach(model.outline) { item in
                        OutlineRow(item: item, isActive: item.index == model.activeHeadingIndex)
                            .contentShape(Rectangle())
                            .onTapGesture { model.jumpToHeading(item.index) }
                    }
                }
                .padding(.vertical, 6)
            }
        }
    }
}

struct OutlineRow: View {
    let item: OutlineItem
    let isActive: Bool

    var body: some View {
        HStack(spacing: 0) {
            if isActive {
                Rectangle()
                    .fill(Color.accentColor)
                    .frame(width: 2)
            } else {
                Color.clear.frame(width: 2)
            }
            Text(item.text)
                .font(.body)
                .fontWeight(isActive ? .semibold : .regular)
                .foregroundStyle(isActive ? Color.primary : Color.secondary)
                .lineLimit(2)
                .padding(.leading, CGFloat(max(item.level - 1, 0)) * 12 + 8)
                .padding(.trailing, 8)
            Spacer()
        }
        .padding(.vertical, 3)
        .background(
            isActive ? Color.accentColor.opacity(0.1) : Color.clear,
            in: Rectangle()
        )
    }
}

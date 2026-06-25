import SwiftUI

struct LibraryView: View {
    @EnvironmentObject private var model: ReaderModel

    private var grouped: [(DayBucket, [DocInfo])] {
        let now = Date()
        let groups = Dictionary(grouping: model.filteredDocs) { DateBuckets.bucket($0.openedAt, now: now) }
        return DayBucket.allCases
            .map { ($0, groups[$0] ?? []) }
            .filter { !$0.1.isEmpty }
    }

    var body: some View {
        VStack(spacing: 0) {
            searchBar
            if model.docs.isEmpty {
                emptyState
            } else {
                docList
            }
        }
    }

    private var searchBar: some View {
        HStack(spacing: 6) {
            Image(systemName: "magnifyingglass").foregroundStyle(.secondary)
            TextField("搜索", text: $model.query)
                .textFieldStyle(.plain)
                .submitLabel(.done)
            if !model.query.isEmpty {
                Button {
                    model.query = ""
                } label: {
                    Image(systemName: "xmark.circle.fill")
                }
                .buttonStyle(.plain)
                .foregroundStyle(.secondary)
            }
        }
        .padding(8)
    }

    private var emptyState: some View {
        VStack(spacing: 8) {
            Spacer()
            Image(systemName: "tray")
                .font(.system(size: 30))
                .foregroundStyle(.secondary)
            Text("还没有缓存的文档").foregroundStyle(.secondary)
            Text("打开或拖入 .md 文件即自动缓存")
                .font(.caption)
                .foregroundStyle(.secondary)
            Spacer()
        }
        .frame(maxWidth: .infinity)
    }

    private var docList: some View {
        List {
            ForEach(grouped, id: \.0) { bucket, docs in
                Section(bucket.title) {
                    ForEach(docs) { doc in
                        docRow(for: doc)
                    }
                }
            }
        }
        .listStyle(.sidebar)
    }

    @ViewBuilder
    private func docRow(for doc: DocInfo) -> some View {
        DocRow(doc: doc, isSelected: doc.id == model.selectedDocID)
            .contentShape(Rectangle())
            .onTapGesture { model.openCached(doc) }
            .contextMenu { contextMenuItems(for: doc) }
    }

    @ViewBuilder
    private func contextMenuItems(for doc: DocInfo) -> some View {
        Button {
            model.toggleFavorite(id: doc.id)
        } label: {
            HStack {
                Image(systemName: doc.favorite ? "star.slash" : "star")
                Text(doc.favorite ? "取消收藏" : "收藏")
            }
        }
        Button(role: .destructive) {
            model.deleteDoc(id: doc.id)
        } label: {
            Label("删除", systemImage: "trash")
        }
    }
}

struct DocRow: View {
    let doc: DocInfo
    let isSelected: Bool

    var body: some View {
        HStack(spacing: 8) {
            if doc.favorite {
                Image(systemName: "star.fill")
                    .foregroundStyle(.yellow)
                    .font(.caption)
            }
            VStack(alignment: .leading, spacing: 2) {
                Text(doc.title)
                    .lineLimit(1)
                    .font(.body)
                Text(DateBuckets.format(doc.openedAt))
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Spacer()
        }
        .padding(.vertical, 3)
        .background(
            isSelected ? Color.accentColor.opacity(0.15) : Color.clear,
            in: RoundedRectangle(cornerRadius: 6)
        )
    }
}

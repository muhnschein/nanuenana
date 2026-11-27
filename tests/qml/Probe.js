// SPDX-License-Identifier: GPL-3.0-or-later
.pragma library

function expect(ok, what) {
    if (!ok)
        console.error("FAILED: " + what)
    return ok
}

function findAll(item, pred, out, seen) {
    seen = seen || []
    if (!item || seen.indexOf(item) >= 0)
        return out
    seen.push(item)
    if (pred(item))
        out.push(item)
    var kids = item.children || []
    for (var i = 0; i < kids.length; ++i)
        findAll(kids[i], pred, out, seen)
    if (item.contentItem)
        findAll(item.contentItem, pred, out, seen)
    return out
}

function named(root, name) {
    return findAll(root, function (x) { return x.objectName === name }, [])
}

function one(root, name) {
    var all = named(root, name)
    return expect(all.length === 1, "one " + name + ": " + all.length) ? all[0] : null
}

function text(root, name) {
    var item = one(root, name)
    return item ? String(item.text) : "<no " + name + ">"
}

function texts(root, name) {
    return named(root, name).map(function (item) { return String(item.text) })
}

function emptyTextTop(shown) {
    var hint = one(shown, "placesEmpty")
    var head = one(shown, "placesHead")
    if (!hint || !head)
        return "<no empty text or header>"
    var top = hint.mapToItem(shown, 0, 0).y
    var free = Math.max(head.mapToItem(shown, 0, head.height).y, shown.height / 3 - hint.height / 2)
    return Math.abs(top - free) < 1 && head.height > 0 && hint.height > 0 ? "clear" : "at " + top + ", free from " + free
}

function coverRoom(cover, lines) {
    var second = one(cover, "coverSecond")
    var more = one(cover, "coverMore")
    if (!second || !more || !more.visible || second.lineCount < 1)
        return null
    var line = second.height / second.lineCount
    cover.actionsHeight = 2 * line
    cover.height = second.mapToItem(cover, 0, 0).y + lines * line + more.height + second.parent.spacing + cover.actionsHeight
    return {
        "lines": second.lineCount,
        "cut": second.truncated,
        "clear": more.mapToItem(cover, 0, more.height).y <= cover.height - cover.actionsHeight
    }
}

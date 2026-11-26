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

// SPDX-License-Identifier: GPL-3.0-or-later
#![recursion_limit = "1024"]
#![allow(non_snake_case)]
#![allow(clippy::useless_transmute)]

pub mod app;
pub mod runtime;

use cpp::cpp;
use nanunana_core::net::{Canned, Fetch, Http};
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

cpp! {{
    #include <QtCore/QCoreApplication>
    #include <QtCore/QLocale>
    #include <QtCore/QTranslator>

    static bool nanunanaInstallTranslations(QCoreApplication *app, const QString &directory)
    {
        QTranslator *english = new QTranslator(app);
        const bool found = english->load(QStringLiteral("harbour-nanunana"), directory);
        if (found)
            app->installTranslator(english);
        QTranslator *local = new QTranslator(app);
        if (local->load(QLocale(), QStringLiteral("harbour-nanunana"), QStringLiteral("-"), directory))
            app->installTranslator(local);
        return found;
    }
}}

cpp! {{
    #include <cstdio>
    #include <cstring>
    #include <csignal>
    #include <execinfo.h>
    #include <sys/prctl.h>
    #include <unistd.h>

    static void nanunanaSay(const char *text)
    {
        const ssize_t written = ::write(STDERR_FILENO, text, std::strlen(text));
        (void)written;
    }

    static void nanunanaCrashTrace(int sig)
    {
        char number[4] = { 0, 0, 0, 0 };
        int at = 0;
        if (sig >= 10)
            number[at++] = char('0' + sig / 10 % 10);
        number[at] = char('0' + sig % 10);
        char thread[17] = { 0 };
        prctl(PR_GET_NAME, thread, 0, 0, 0);
        nanunanaSay("CRASH   signal ");
        nanunanaSay(number);
        nanunanaSay(" in thread ");
        nanunanaSay(thread);
        nanunanaSay("\n");
        void *frames[64];
        backtrace_symbols_fd(frames, backtrace(frames, 64), STDERR_FILENO);
        std::raise(sig);
    }

    static void nanunanaReportCrashes()
    {
        std::setvbuf(stdout, nullptr, _IOLBF, 0);
        void *warm[1];
        backtrace(warm, 1);
        struct sigaction action;
        std::memset(&action, 0, sizeof action);
        action.sa_handler = nanunanaCrashTrace;
        action.sa_flags = SA_RESETHAND | SA_ONSTACK;
        sigemptyset(&action.sa_mask);
        for (int sig : { SIGSEGV, SIGBUS, SIGILL, SIGFPE, SIGABRT })
            sigaction(sig, &action, nullptr);
    }
}}

cpp! {{
    #include <QtGui/QGuiApplication>
    #include <QtQuick/QQuickView>
    #include <QtQml/QQmlEngine>
    #include <QtQml/QQmlContext>
    #include <QtCore/QUrl>
    #include <QtCore/QElapsedTimer>
    #include <QtCore/QFileInfo>
    #include <QtCore/QEvent>
    #include <QtCore/QEventLoop>
    #include <QtQml/QQmlComponent>
    #include <QtCore/QLibraryInfo>
    #include <QtCore/QRegularExpression>
    #include <memory>
#ifdef NANUNANA_SAILFISH
    #include <sailfishapp.h>
#endif

    static QStringList *nanunanaCheckProblems = nullptr;
    static QString nanunanaCheckFile;

    static void nanunanaCheckMessage(QtMsgType type, const QMessageLogContext &context, const QString &msg)
    {
        const QString where = QString::fromUtf8(context.file ? context.file : "");
        const bool ours = where.contains(nanunanaCheckFile) || msg.contains(nanunanaCheckFile);
        const bool reported = msg.startsWith(QLatin1String("FAILED: "));
        if (nanunanaCheckProblems && type != QtDebugMsg && type != QtInfoMsg && (ours || reported))
            *nanunanaCheckProblems << (where + QStringLiteral(": ") + msg);
        else
            std::fprintf(stderr, "note    %s\n", qPrintable(msg));
    }

    static bool nanunanaThrown(const QQmlError &e)
    {
        static const QRegularExpression thrown(QStringLiteral("^[A-Za-z]*Error: "));
        static const QString qt = QLibraryInfo::location(QLibraryInfo::Qml2ImportsPath) + QLatin1Char('/');
        return thrown.match(e.description()).hasMatch() && !e.url().toLocalFile().startsWith(qt);
    }

    static int nanunanaQmlCheck(int argc, char **argv)
    {
        static int count = 0;
        count = argc;
        QGuiApplication app(count, argv);
        QStringList importPaths;
        QStringList files;
        QString translations;
        const QStringList args = QCoreApplication::arguments().mid(2);
        for (int i = 0; i < args.size(); ++i) {
            if (args.at(i) == QLatin1String("-I") && i + 1 < args.size())
                importPaths << args.at(++i);
            else if (args.at(i) == QLatin1String("-T") && i + 1 < args.size())
                translations = args.at(++i);
            else if (args.at(i) == QLatin1String("-D") && i + 1 < args.size())
                ++i;
            else
                files << args.at(i);
        }
        if (!translations.isEmpty() && !nanunanaInstallTranslations(&app, translations)) {
            std::printf("ERROR   no harbour-nanunana.qm in %s\n", qPrintable(translations));
            return 1;
        }
        int failures = 0;
        for (const QString &file : files) {
            QQmlEngine engine;
            for (const QString &path : importPaths)
                engine.addImportPath(path);
            QStringList problems;
            QObject::connect(&engine, &QQmlEngine::warnings, [&problems, &file](const QList<QQmlError> &list) {
                for (const QQmlError &e : list) {
                    if (e.url().toLocalFile() == file || e.toString().contains(file) || nanunanaThrown(e))
                        problems << e.toString();
                    else
                        std::printf("note    %s\n", qPrintable(e.toString()));
                }
            });
            nanunanaCheckProblems = &problems;
            nanunanaCheckFile = file;
            QtMessageHandler previous = qInstallMessageHandler(nanunanaCheckMessage);
            QQmlComponent component(&engine, QUrl::fromLocalFile(file), QQmlComponent::PreferSynchronous);
            while (component.isLoading())
                QCoreApplication::processEvents();
            std::unique_ptr<QObject> object(component.isReady() ? component.create() : nullptr);
            for (int spin = 0; spin < 10; ++spin)
                QCoreApplication::processEvents();
            if (object && object->property("scenarioRunning").isValid()) {
                QElapsedTimer clock;
                clock.start();
                while (object->property("scenarioRunning").toBool() && clock.elapsed() < 60000) {
                    QCoreApplication::processEvents(QEventLoop::AllEvents, 20);
                    QCoreApplication::sendPostedEvents(nullptr, QEvent::DeferredDelete);
                }
                if (object->property("scenarioRunning").toBool())
                    problems << QStringLiteral("scenario still running after 60 s");
            }
            if (object && QFileInfo(file).fileName().startsWith(QLatin1String("tst_"))
                && !object->property("reachedEnd").toBool())
                problems << QStringLiteral("checks unproven: root reachedEnd is not true");
            for (const QQmlError &e : component.errors())
                problems << e.toString();
            if (!object && problems.isEmpty())
                problems << QStringLiteral("could not be created");
            qInstallMessageHandler(previous);
            nanunanaCheckProblems = nullptr;
            for (const QString &p : problems)
                std::printf("ERROR   %s\n", qPrintable(p));
            std::printf("%s %s\n", problems.isEmpty() ? "ok     " : "FAILED ", qPrintable(file));
            if (!problems.isEmpty())
                ++failures;
        }
        std::printf("%d of %d files failed\n", failures, static_cast<int>(files.size()));
        return failures ? 1 : 0;
    }
}}

fn canned_dir(args: &[String]) -> String {
    args.iter()
        .position(|a| a == "-D")
        .and_then(|at| args.get(at + 1))
        .cloned()
        .unwrap_or_default()
}

fn is_check(args: &[String]) -> bool {
    args.get(1).is_some_and(|a| a == "--qml-check")
}

fn fetch_for(args: &[String]) -> Box<dyn Fetch> {
    if is_check(args) {
        Box::new(Canned::new(canned_dir(args)))
    } else {
        Box::new(Http::new())
    }
}

/// # Safety
/// `argc`/`argv` must be the process arguments as passed to `main`.
pub unsafe fn run(argc: c_int, argv: *mut *mut c_char) -> c_int {
    let args: Vec<String> = (0..usize::try_from(argc).unwrap_or_default())
        .map(|i| CStr::from_ptr(*argv.add(i)).to_string_lossy().into_owned())
        .collect();
    runtime::open(nanunana_core::paths::data_dir(), fetch_for(&args));
    register_types();
    cpp!([argc as "int", argv as "char**"] -> c_int as "int" {
            if (argc > 1 && qstrcmp(argv[1], "--qml-check") == 0) {
                nanunanaReportCrashes();
                return nanunanaQmlCheck(argc, argv);
            }
            static int count = 0;
            count = argc;
    #ifdef NANUNANA_SAILFISH
            QGuiApplication *app = SailfishApp::application(count, argv);
            nanunanaInstallTranslations(app, SailfishApp::pathTo(QStringLiteral("translations")).toLocalFile());
            QQuickView *view = SailfishApp::createView();
            view->setSource(SailfishApp::pathToMainQml());
            view->show();
    #else
            QGuiApplication *app = new QGuiApplication(count, argv);
            QQuickView *view = new QQuickView();
            QByteArray qml = qgetenv("NANUNANA_QML");
            view->setSource(QUrl::fromLocalFile(QString::fromLocal8Bit(qml)));
            view->show();
    #endif
            int rc = app->exec();
            delete view;
            delete app;
            return rc;
        })
}

pub const QML_URI: &str = "Nanunana";

pub fn register_types() {
    qmetaobject::qml_register_singleton_type::<app::App>(&cstr(QML_URI), 1, 0, &cstr("App"));
}

pub fn cstr(s: &str) -> std::ffi::CString {
    std::ffi::CString::new(s).unwrap_or_default()
}

pub fn system_locale() -> String {
    let name = cpp!(unsafe [] -> qmetaobject::QString as "QString" { return QLocale().name(); });
    name.to_string()
}

pub fn report_crashes() {
    cpp!(unsafe [] { nanunanaReportCrashes(); })
}

pub fn qml_check(args: &[String]) -> c_int {
    register_types();
    let mut owned: Vec<std::ffi::CString> = vec![cstr("harbour-nanunana"), cstr("--qml-check")];
    owned.extend(args.iter().map(|a| cstr(a)));
    let mut ptrs: Vec<*mut c_char> = owned.iter().map(|c| c.as_ptr() as *mut c_char).collect();
    ptrs.push(std::ptr::null_mut());
    let argc = c_int::try_from(owned.len()).unwrap_or(c_int::MAX);
    let argv = ptrs.as_mut_ptr();
    unsafe {
        cpp!([argc as "int", argv as "char**"] -> c_int as "int" { return nanunanaQmlCheck(argc, argv); })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|a| (*a).to_owned()).collect()
    }

    #[test]
    fn the_check_mode_is_the_first_argument_only() {
        assert!(is_check(&args(&["app", "--qml-check", "a.qml"])));
        assert!(!is_check(&args(&["app"])));
        assert!(!is_check(&args(&["app", "a.qml", "--qml-check"])));
    }

    #[test]
    fn the_canned_folder_follows_its_switch() {
        assert_eq!(
            canned_dir(&args(&["app", "--qml-check", "-D", "/data", "a.qml"])),
            "/data"
        );
        assert_eq!(canned_dir(&args(&["app", "--qml-check", "a.qml"])), "");
        assert_eq!(canned_dir(&args(&["app", "--qml-check", "-D"])), "");
    }
}

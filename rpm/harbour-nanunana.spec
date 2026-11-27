# SPDX-License-Identifier: GPL-3.0-or-later

Name:       harbour-nanunana
Summary:    Unofficial warning app
Version:    0.1.0
Release:    1
License:    GPLv3+ and MIT and ASL 2.0 and BSD and ISC and CC-BY
URL:        https://github.com/muhnschein/nanuenana
Source0:    %{name}-%{version}.tar.bz2
ExclusiveArch: aarch64

BuildRequires: pkgconfig(Qt5Core)
BuildRequires: pkgconfig(Qt5Qml)
BuildRequires: pkgconfig(Qt5Quick)
BuildRequires: pkgconfig(sailfishapp)
BuildRequires: qt5-qttools-linguist
BuildRequires: desktop-file-utils

Requires:   sailfishsilica-qt5 >= 0.10.9
Requires:   nemo-qml-plugin-notifications-qt5
Requires:   nemo-qml-plugin-dbus-qt5
Requires:   qml(Nemo.KeepAlive)
Requires:   qml(QtPositioning)
Requires:   qt5-qtsvg-plugin-imageformat-svg

%define debug_package %{nil}
%define __provides_exclude_from ^%{_datadir}/.*$

%description
Nanünana shows official warnings for places you choose and for where you
are: civil protection, police, weather and flood warnings in Germany. It
is not an official app. In an emergency follow sirens, cell broadcast,
radio and the authorities first.

%prep
%setup -q -n %{name}-%{version}

%build
test -x prebuilt/harbour-nanunana || { echo "prebuilt/harbour-nanunana missing: run tools/ci/cross-build.sh" >&2; exit 1; }
lrelease -idbased translations/*.ts

%install
install -D -m 0755 prebuilt/harbour-nanunana %{buildroot}%{_bindir}/%{name}
install -d %{buildroot}%{_datadir}/%{name}/qml %{buildroot}%{_datadir}/%{name}/translations
cp -r qml/* %{buildroot}%{_datadir}/%{name}/qml/
install -m 0644 translations/*.qm %{buildroot}%{_datadir}/%{name}/translations/
install -m 0644 LICENSE %{buildroot}%{_datadir}/%{name}/LICENSE
for size in 86x86 108x108 128x128 172x172; do
    install -D -m 0644 icons/$size/%{name}.png %{buildroot}%{_datadir}/icons/hicolor/$size/apps/%{name}.png
done
install -D -m 0644 %{name}.desktop %{buildroot}%{_datadir}/applications/%{name}.desktop
desktop-file-validate %{buildroot}%{_datadir}/applications/%{name}.desktop || :

%files
%defattr(-,root,root,-)
%{_bindir}/%{name}
%{_datadir}/%{name}
%{_datadir}/applications/%{name}.desktop
%{_datadir}/icons/hicolor/*/apps/%{name}.png

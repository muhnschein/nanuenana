# Nanünana

An unofficial warning app for Sailfish OS. It shows the warnings that authorities publish for places you choose and, if you turn it on, for where you are: civil protection, police, weather and flood warnings.

**Nanünana is not an official app.** Volunteers make it. No authority runs, checks or endorses it. It can be late and it can miss a warning. In an emergency follow the official channels first: sirens, cell broadcast on your phone, radio and television, and what the authorities on site tell you.

## What it does

- Warnings for saved places, each with a distance of 1 to 25 km or its whole district, and for your current location.
- Notifications for new warnings, by category and level, and when a warning ends.
- Says so clearly when it could not check: a place it could not update never shows "No warnings".
- Warning texts are shown unchanged, in the language the issuer provides.
- Places, settings and your position stay on the phone. No account, no tracking.

It checks for warnings every 5 to 30 minutes, only while it is open or minimised. If you close it, you get no alerts.

## Where it works

Germany, with warnings from the Bundesamt für Bevölkerungsschutz und Katastrophenhilfe ([warnung.bund.de](https://warnung.bund.de)). More countries follow one by one.

Nanünana is not offered in the Russian Federation, in Belarus, or in territories under Russian control outside Russia. It is not offered in Iran, North Korea or Syria.

## Install

Sailfish OS 5.2 or newer, aarch64. Builds are attached to the CI runs of this repository; a release in the Jolla Store is planned.

## Build

```
./tools/ci/install-host-deps.sh nextest cross
make check
make rpm
make check-rpm
```

`make rpm` needs Docker and builds the package with the Sailfish SDK. `make check-rpm` runs the Harbour validator and the page tests on the SDK target. More in [doc/CONTRIBUTING-CODE.md](doc/CONTRIBUTING-CODE.md).

## Sources and licence

Warnings: Bundesamt für Bevölkerungsschutz und Katastrophenhilfe, with warnings of the civil protection authorities of the Länder and districts, the police forces of the Länder, Deutscher Wetterdienst and the Länderübergreifendes Hochwasserportal.

Place names: [GeoNames](https://www.geonames.org), CC BY 4.0.

Nanünana is free software under the GNU General Public License, version 3 or later. See [LICENSE](LICENSE).

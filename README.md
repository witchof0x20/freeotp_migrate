# How to use
## Install ADB

## Get FreeOTP's files using an ADB backup
Use ADB to trigger a backup for FreeOTP's files
```bash
adb backup ~/freeotp.ab org.fedorahosted.freeotp
```
* Unlock your phone and allow the backup. This may require a password. If so, choose a password
## Decrypt the ADB backup
Clone a backup extractor tool
```bash
git clone https://github.com/nelenkov/android-backup-extractor
```
Build it
```bash
./gradlew
```
Run it with
```bash
java -jar build/libs/abe-all.jar unpack ~/freeotp.ab ~/freeotp.tar
```
Type in the password you set if you set one

Extract the specific file
```bash
tar xvf ~/freeotp.tar apps/org.fedorahosted.freeotp/sp/tokens.xml
mv apps/org.fedorahosted.freeotp/sp/tokens.xml ~/tokens.xml
rm -rf apps
```
You now have a tokens.xml file that can be used with this program
## Using this program
```bash
git clone https://github.com/witchof0x20/freeotp_migrate
cd freeotp_migrate
```
### QR codes
If you want QR codes for each token, use these steps
```bash
mkdir ~/qr
cargo run --release -- -i ~/tokens.xml -q -o ~/qr
```
### URIs
If you just want the URI for each token, use this command
```bash
cargo run --release -- -i ~/tokens.xml
```

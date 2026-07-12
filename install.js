const fs = require('fs');
const https = require('https');
const path = require('path');
const os = require('os');

const VERSION = 'v1.0.0';
const REPO = 'bijanmurmu/chronx';

const platform = os.platform();
const arch = os.arch();

let assetName = '';
let binName = 'chronx';

if (platform === 'win32') {
    assetName = 'chronx-windows-x86_64.exe';
    binName = 'chronx.exe';
} else if (platform === 'darwin') {
    assetName = arch === 'arm64' ? 'chronx-macos-aarch64' : 'chronx-macos-x86_64';
} else if (platform === 'linux') {
    assetName = 'chronx-linux-x86_64';
} else {
    console.error(`Unsupported platform: ${platform}`);
    process.exit(1);
}

const url = `https://github.com/${REPO}/releases/download/${VERSION}/${assetName}`;
const binPath = path.join(__dirname, 'bin');
const exePath = path.join(binPath, binName);

if (!fs.existsSync(binPath)) {
    fs.mkdirSync(binPath);
}

console.log(`Downloading chronx ${VERSION} for ${platform} from ${url}...`);

function download(url, dest) {
    return new Promise((resolve, reject) => {
        https.get(url, (res) => {
            if (res.statusCode === 301 || res.statusCode === 302) {
                return download(res.headers.location, dest).then(resolve).catch(reject);
            }
            if (res.statusCode !== 200) {
                return reject(new Error(`Failed to download: ${res.statusCode}`));
            }
            const file = fs.createWriteStream(dest);
            res.pipe(file);
            file.on('finish', () => {
                file.close();
                resolve();
            });
        }).on('error', reject);
    });
}

download(url, exePath)
    .then(() => {
        if (platform !== 'win32') {
            fs.chmodSync(exePath, 0o755);
        }
        console.log('Successfully installed chronx!');
    })
    .catch((err) => {
        console.error('Installation failed:', err.message);
        process.exit(1);
    });

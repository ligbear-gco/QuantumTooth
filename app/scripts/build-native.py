#!/usr/bin/env python3
"""@file @brief Build pinned static Android libcrypto and the independent locked JNI crate."""
import fcntl
import hashlib
import os
from pathlib import Path
import shutil
import subprocess
import tarfile
import urllib.request

ROOT = Path(__file__).resolve().parents[2]
WORK = ROOT / 'local/g2-3-react-native-debug-app/native-build'
VERSION = '3.5.9'
SHA256 = '603f5602e2eef00d77fbd429d34dcd5822bb301757a1bc9cdb24c670f1eb859a'
NDK_VERSION = '27.1.12297006'


def run(args, cwd, env):
    """@brief Run one build operation; fail without ever reading or printing pair bytes."""
    subprocess.run(args, cwd=cwd, env=env, check=True)


def main():
    """@brief Validate provisioning first, verify sources, and compile only ARM64 Android."""
    pair = Path(os.environ.get('QTOOTH_PAIR_FILE', ROOT / 'local/ble/pair.bin'))
    if not pair.is_file() or pair.stat().st_size != 64:
        raise SystemExit('Restore the existing 64-byte private pair at local/ble/pair.bin')
    sdk = os.environ.get('ANDROID_HOME') or os.environ.get('ANDROID_SDK_ROOT')
    if not sdk:
        raise SystemExit('Set ANDROID_HOME to the installed Android SDK')
    ndk = Path(sdk) / 'ndk' / NDK_VERSION
    bin_dir = ndk / 'toolchains/llvm/prebuilt/linux-x86_64/bin'
    if not (bin_dir / 'aarch64-linux-android24-clang').is_file():
        raise SystemExit(f'Install NDK {NDK_VERSION} for Linux x86_64')
    WORK.mkdir(parents=True, exist_ok=True, mode=0o700)
    os.umask(0o077)
    with (WORK / 'build.lock').open('w') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        archive = WORK / f'openssl-{VERSION}.tar.gz'
        cached = ROOT / 'cache/downloads' / archive.name
        if not archive.exists():
            if cached.is_file():
                shutil.copyfile(cached, archive)
            else:
                urllib.request.urlretrieve(f'https://github.com/openssl/openssl/releases/download/openssl-{VERSION}/{archive.name}', archive)
        if hashlib.sha256(archive.read_bytes()).hexdigest() != SHA256:
            raise SystemExit('OpenSSL source checksum mismatch')
        env = dict(os.environ)
        env.update(ANDROID_NDK_ROOT=str(ndk), PATH=str(bin_dir) + os.pathsep + env['PATH'])
        source = WORK / f'openssl-{VERSION}'
        install = WORK / 'openssl-install'
        stamp = WORK / 'openssl.stamp'
        signature = f'{SHA256}:ndk={ndk}:api=24:pic:static:no-module:v1'
        if not stamp.exists() or stamp.read_text() != signature or not (install / 'lib/libcrypto.a').exists():
            if source.exists():
                shutil.rmtree(source)
            with tarfile.open(archive) as bundle:
                bundle.extractall(WORK, filter='data')
            run(['perl', 'Configure', 'android-arm64', 'no-shared', 'no-tests', 'no-module',
                 'no-apps', 'no-docs', '-fPIC', '-D__ANDROID_API__=24',
                 f'-ffile-prefix-map={ROOT}=.', f'--prefix={install}', '--libdir=lib'], source, env)
            run(['make', '-j', str(min(os.cpu_count() or 2, 8)), 'build_libs'], source, env)
            run(['make', 'install_dev'], source, env)
            stamp.write_text(signature)
        env.update(OPENSSL_DIR=str(install), OPENSSL_STATIC='1', OPENSSL_NO_VENDOR='1',
                   QTOOTH_PAIR_FILE=str(pair.resolve()),
                   CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=str(bin_dir / 'aarch64-linux-android24-clang'),
                   CC_aarch64_linux_android=str(bin_dir / 'aarch64-linux-android24-clang'),
                   AR_aarch64_linux_android=str(bin_dir / 'llvm-ar'),
                   CARGO_TARGET_AARCH64_LINUX_ANDROID_RUSTFLAGS='-C link-arg=-Wl,-z,max-page-size=16384 -C link-arg=-Wl,-z,common-page-size=16384 -C link-arg=-Wl,--exclude-libs,ALL')
        run(['cargo', 'build', '--locked', '--release', '--target', 'aarch64-linux-android'], ROOT / 'app/native', env)
        output = ROOT / 'app/android/app/build/generated/qtoothJni/arm64-v8a'
        output.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(ROOT / 'app/native/target/aarch64-linux-android/release/libqtooth_android.so', output / 'libqtooth_android.so')


if __name__ == '__main__':
    main()

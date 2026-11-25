# @file
# @brief Locked crate fetch inputs generated from components/ble/Cargo.lock.
# Regenerate with bitbake -c update_crates qtooth-ble.
SRC_URI += " \
    crate://crates.io/anstream/1.0.0 \
    crate://crates.io/anstyle/1.0.14 \
    crate://crates.io/anstyle-parse/1.0.0 \
    crate://crates.io/anstyle-query/1.1.5 \
    crate://crates.io/anstyle-wincon/3.0.11 \
    crate://crates.io/autocfg/1.5.1 \
    crate://crates.io/bitflags/2.13.2 \
    crate://crates.io/bluer/0.17.4 \
    crate://crates.io/bumpalo/3.20.4 \
    crate://crates.io/bytes/1.12.2 \
    crate://crates.io/cc/1.11.3 \
    crate://crates.io/cfg-if/1.0.5 \
    crate://crates.io/cfg_aliases/0.2.2 \
    crate://crates.io/clap/4.6.9 \
    crate://crates.io/clap_builder/4.6.7 \
    crate://crates.io/clap_derive/4.6.9 \
    crate://crates.io/clap_lex/1.1.2 \
    crate://crates.io/colorchoice/1.0.5 \
    crate://crates.io/custom_debug/0.6.2 \
    crate://crates.io/custom_debug_derive/0.6.2 \
    crate://crates.io/darling/0.20.11 \
    crate://crates.io/darling_core/0.20.11 \
    crate://crates.io/darling_macro/0.20.11 \
    crate://crates.io/dbus/0.9.12 \
    crate://crates.io/dbus-crossroads/0.5.3 \
    crate://crates.io/dbus-tokio/0.7.6 \
    crate://crates.io/displaydoc/0.2.7 \
    crate://crates.io/errno/0.3.15 \
    crate://crates.io/find-msvc-tools/0.1.15 \
    crate://crates.io/fnv/1.0.7 \
    crate://crates.io/foreign-types/0.3.2 \
    crate://crates.io/foreign-types-shared/0.1.1 \
    crate://crates.io/futures/0.3.34 \
    crate://crates.io/futures-channel/0.3.34 \
    crate://crates.io/futures-core/0.3.34 \
    crate://crates.io/futures-executor/0.3.34 \
    crate://crates.io/futures-io/0.3.34 \
    crate://crates.io/futures-macro/0.3.34 \
    crate://crates.io/futures-sink/0.3.34 \
    crate://crates.io/futures-task/0.3.34 \
    crate://crates.io/futures-util/0.3.34 \
    crate://crates.io/getrandom/0.4.3 \
    crate://crates.io/heck/0.5.0 \
    crate://crates.io/hex/0.4.3 \
    crate://crates.io/ident_case/1.0.1 \
    crate://crates.io/is_terminal_polyfill/1.70.2 \
    crate://crates.io/itoa/1.0.19 \
    crate://crates.io/js-sys/0.3.107 \
    crate://crates.io/lazy_static/1.5.1 \
    crate://crates.io/libc/0.2.191 \
    crate://crates.io/libdbus-sys/0.2.7 \
    crate://crates.io/log/0.4.35 \
    crate://crates.io/macaddr/1.0.1 \
    crate://crates.io/memchr/2.8.3 \
    crate://crates.io/mio/1.2.5 \
    crate://crates.io/nix/0.29.0 \
    crate://crates.io/num-derive/0.4.2 \
    crate://crates.io/num-traits/0.2.19 \
    crate://crates.io/once_cell_polyfill/1.70.2 \
    crate://crates.io/openssl/0.10.82 \
    crate://crates.io/openssl-macros/0.1.1 \
    crate://crates.io/openssl-sys/0.9.118 \
    crate://crates.io/pin-project/1.2.0 \
    crate://crates.io/pin-project-internal/1.2.0 \
    crate://crates.io/pin-project-lite/0.2.18 \
    crate://crates.io/pkg-config/0.3.34 \
    crate://crates.io/proc-macro2/1.0.107 \
    crate://crates.io/quote/1.0.47 \
    crate://crates.io/r-efi/6.0.0 \
    crate://crates.io/rustversion/1.0.23 \
    crate://crates.io/serde/1.0.229 \
    crate://crates.io/serde_core/1.0.229 \
    crate://crates.io/serde_derive/1.0.229 \
    crate://crates.io/serde_json/1.0.154 \
    crate://crates.io/shlex/2.0.1 \
    crate://crates.io/signal-hook-registry/1.4.8 \
    crate://crates.io/slab/0.4.12 \
    crate://crates.io/socket2/0.6.5 \
    crate://crates.io/strsim/0.11.1 \
    crate://crates.io/strum/0.26.3 \
    crate://crates.io/strum_macros/0.26.4 \
    crate://crates.io/syn/2.0.119 \
    crate://crates.io/syn/3.0.10 \
    crate://crates.io/synstructure/0.13.2 \
    crate://crates.io/tokio/1.54.1 \
    crate://crates.io/tokio-macros/2.7.2 \
    crate://crates.io/tokio-stream/0.1.19 \
    crate://crates.io/unicode-ident/1.0.27 \
    crate://crates.io/utf8parse/0.2.2 \
    crate://crates.io/uuid/1.29.3 \
    crate://crates.io/vcpkg/0.2.15 \
    crate://crates.io/wasi/0.11.1+wasi-snapshot-preview1 \
    crate://crates.io/wasm-bindgen/0.2.130 \
    crate://crates.io/wasm-bindgen-macro/0.2.130 \
    crate://crates.io/wasm-bindgen-macro-support/0.2.130 \
    crate://crates.io/wasm-bindgen-shared/0.2.130 \
    crate://crates.io/windows-link/0.2.1 \
    crate://crates.io/windows-sys/0.61.2 \
    crate://crates.io/zeroize/1.9.2 \
    crate://crates.io/zmij/1.0.23 \
"

SRC_URI[anstream-1.0.0.sha256sum] = "824a212faf96e9acacdbd09febd34438f8f711fb84e09a8916013cd7815ca28d"
SRC_URI[anstyle-1.0.14.sha256sum] = "940b3a0ca603d1eade50a4846a2afffd5ef57a9feac2c0e2ec2e14f9ead76000"
SRC_URI[anstyle-parse-1.0.0.sha256sum] = "52ce7f38b242319f7cabaa6813055467063ecdc9d355bbb4ce0c68908cd8130e"
SRC_URI[anstyle-query-1.1.5.sha256sum] = "40c48f72fd53cd289104fc64099abca73db4166ad86ea0b4341abe65af83dadc"
SRC_URI[anstyle-wincon-3.0.11.sha256sum] = "291e6a250ff86cd4a820112fb8898808a366d8f9f58ce16d1f538353ad55747d"
SRC_URI[autocfg-1.5.1.sha256sum] = "f2032f911046de80f0a198e0901378627c33f59ea0ac00e363d481118bd70a53"
SRC_URI[bitflags-2.13.2.sha256sum] = "3ded4057c258ba199e2d26386d3af3780957ecaee6c4ef4041c6b4b8b97c0b06"
SRC_URI[bluer-0.17.4.sha256sum] = "af68112f5c60196495c8b0eea68349817855f565df5b04b2477916d09fb1a901"
SRC_URI[bumpalo-3.20.4.sha256sum] = "87a0cf66fd1eb2fb7b822cb78370d82303750809bc6ba855f3f58e517d3238db"
SRC_URI[bytes-1.12.2.sha256sum] = "776dfebd015e2d8732c684a2c71003674ccb49457b539b9e82151ba38e96dd7c"
SRC_URI[cc-1.11.3.sha256sum] = "02f0e023adb368345ce0ca6cdfc0f24a06554468d0e8503e14a0841de79bfa69"
SRC_URI[cfg-if-1.0.5.sha256sum] = "4e7648175b45a9a48536d676f68d918270699102aa8dab5496df06904c914600"
SRC_URI[cfg_aliases-0.2.2.sha256sum] = "f079e83a288787bcd14a6aea84cee5c87a67c5a3e660c30f557a3d24761b3527"
SRC_URI[clap-4.6.9.sha256sum] = "844c5c0b65be192cb68d570cca1b21a6444431d24e39c084dc0336177015337c"
SRC_URI[clap_builder-4.6.7.sha256sum] = "ec0797fb7aeb1406c84efac526901f7ec3ead2124f946b494e72879d4b54704d"
SRC_URI[clap_derive-4.6.9.sha256sum] = "b4c174f11f949b9ca8c60580925e282b9cbd5454750e506f08648b2038080cd7"
SRC_URI[clap_lex-1.1.2.sha256sum] = "e0921b3876b3a1df8a1c9f912e7f94342458022c2e84288015893d83c4d62a79"
SRC_URI[colorchoice-1.0.5.sha256sum] = "1d07550c9036bf2ae0c684c4297d503f838287c83c53686d05370d0e139ae570"
SRC_URI[custom_debug-0.6.2.sha256sum] = "2da7d1ad9567b3e11e877f1d7a0fa0360f04162f94965fc4448fbed41a65298e"
SRC_URI[custom_debug_derive-0.6.2.sha256sum] = "a707ceda8652f6c7624f2be725652e9524c815bf3b9d55a0b2320be2303f9c11"
SRC_URI[darling-0.20.11.sha256sum] = "fc7f46116c46ff9ab3eb1597a45688b6715c6e628b5c133e288e709a29bcb4ee"
SRC_URI[darling_core-0.20.11.sha256sum] = "0d00b9596d185e565c2207a0b01f8bd1a135483d02d9b7b0a54b11da8d53412e"
SRC_URI[darling_macro-0.20.11.sha256sum] = "fc34b93ccb385b40dc71c6fceac4b2ad23662c7eeb248cf10d529b7e055b6ead"
SRC_URI[dbus-0.9.12.sha256sum] = "3ab69f03cc8c4340c9c8e315114e1658e6775a9b16a04357973aa21cec22b32e"
SRC_URI[dbus-crossroads-0.5.3.sha256sum] = "64bff0bd181fba667660276c6b7ebdc50cff37ce593e7adf9e734f89c8f444e8"
SRC_URI[dbus-tokio-0.7.6.sha256sum] = "007688d459bc677131c063a3a77fb899526e17b7980f390b69644bdbc41fad13"
SRC_URI[displaydoc-0.2.7.sha256sum] = "c6232dd377dcc64799954cbd3a9bb882e9cdc1308ccd87b1c098f1fb2eaf82a8"
SRC_URI[errno-0.3.15.sha256sum] = "e35d6e78f4cebb90e51d543db8fe41528851d6deeef6ef21ebd4bfdc1e784776"
SRC_URI[find-msvc-tools-0.1.15.sha256sum] = "1c66915cc5391097de7a2b0b972e8505aa33c0df401fd3a6811dbdb32881ba48"
SRC_URI[fnv-1.0.7.sha256sum] = "3f9eec918d3f24069decb9af1554cad7c880e2da24a9afd88aca000531ab82c1"
SRC_URI[foreign-types-0.3.2.sha256sum] = "f6f339eb8adc052cd2ca78910fda869aefa38d22d5cb648e6485e4d3fc06f3b1"
SRC_URI[foreign-types-shared-0.1.1.sha256sum] = "00b0228411908ca8685dba7fc2cdd70ec9990a6e753e89b6ac91a84c40fbaf4b"
SRC_URI[futures-0.3.34.sha256sum] = "9a31d2a3fbaaeb2af2368bbdd904aa8e812d3c04a1ee10d3171f52d556e5d0a3"
SRC_URI[futures-channel-0.3.34.sha256sum] = "b1f9e3d69d39e4862ffed03ed071a76f9a13ba1d9109d355b0f0aa6b15e393c4"
SRC_URI[futures-core-0.3.34.sha256sum] = "92d699e522242e69e3003b94ecc1f960f3a5e015aa7c5d7486e65ad01dd94f5e"
SRC_URI[futures-executor-0.3.34.sha256sum] = "031b47cf1a3c6cc8bc2fc76cd437f521619387907d469316e7c0bc278f1f5432"
SRC_URI[futures-io-0.3.34.sha256sum] = "53c0fa8157de1303bfffdaa1cc2a673bfffb60102f76b0ef4441659124373fed"
SRC_URI[futures-macro-0.3.34.sha256sum] = "9fb9654ba8355388abeb8dcb4fc62f511300867002afc858860463bdd9fe0c44"
SRC_URI[futures-sink-0.3.34.sha256sum] = "1944426bf7d03f1d14f708785e4b33efd750b36d48a157b836b3efc15ede8e1d"
SRC_URI[futures-task-0.3.34.sha256sum] = "cd417de3d1d015fc3bfd2b1ea46dfc7bab72ef86f1cc7cc9c78e728b34a6d1fd"
SRC_URI[futures-util-0.3.34.sha256sum] = "0d50a92467f8ba5dd6e3ee5d4bd04d73ab2e4e1c44474a0674821dfce14b79bc"
SRC_URI[getrandom-0.4.3.sha256sum] = "300e883d756b2e4ec94e02791f39b04b522276138852cfc41d9fb7e904106099"
SRC_URI[heck-0.5.0.sha256sum] = "2304e00983f87ffb38b55b444b5e3b60a884b5d30c0fca7d82fe33449bbe55ea"
SRC_URI[hex-0.4.3.sha256sum] = "7f24254aa9a54b5c858eaee2f5bccdb46aaf0e486a595ed5fd8f86ba55232a70"
SRC_URI[ident_case-1.0.1.sha256sum] = "b9e0384b61958566e926dc50660321d12159025e767c18e043daf26b70104c39"
SRC_URI[is_terminal_polyfill-1.70.2.sha256sum] = "a6cb138bb79a146c1bd460005623e142ef0181e3d0219cb493e02f7d08a35695"
SRC_URI[itoa-1.0.19.sha256sum] = "3bd1574e617a9a6c1f8f8181bd7e841e1f20e4277607ef28945765188d6284d4"
SRC_URI[js-sys-0.3.107.sha256sum] = "8dc09d2154098bbf4ef54eab6b117a3c24cbbffe94046b16ee6d6d4510584d3a"
SRC_URI[lazy_static-1.5.1.sha256sum] = "20870f649af7073d53e38067b2a84312175d56ea15217e1b15bc83506ec50afb"
SRC_URI[libc-0.2.191.sha256sum] = "65cd87c9a6d60536580b316ad985726104730a5151c99845f2490428ef91240a"
SRC_URI[libdbus-sys-0.2.7.sha256sum] = "328c4789d42200f1eeec05bd86c9c13c7f091d2ba9a6ea35acdf51f31bc0f043"
SRC_URI[log-0.4.35.sha256sum] = "5d15f86d12da44cd8d4ec3c73984537eb7f4cadcb9776a575a0a8a05ca11005c"
SRC_URI[macaddr-1.0.1.sha256sum] = "baee0bbc17ce759db233beb01648088061bf678383130602a298e6998eedb2d8"
SRC_URI[memchr-2.8.3.sha256sum] = "cf8baf1c55e62ffcace7a9f06f4bd9cd3f0c4beb022d3b367256b91b87513d98"
SRC_URI[mio-1.2.5.sha256sum] = "20521a903f02b0a573cd6840e9d0a5a0cc043d361fe924092cd70f8556cc9c4a"
SRC_URI[nix-0.29.0.sha256sum] = "71e2746dc3a24dd78b3cfcb7be93368c6de9963d30f43a6a73998a9cf4b17b46"
SRC_URI[num-derive-0.4.2.sha256sum] = "ed3955f1a9c7c0c15e092f9c887db08b1fc683305fdf6eb6684f22555355e202"
SRC_URI[num-traits-0.2.19.sha256sum] = "071dfc062690e90b734c0b2273ce72ad0ffa95f0c74596bc250dcfd960262841"
SRC_URI[once_cell_polyfill-1.70.2.sha256sum] = "384b8ab6d37215f3c5301a95a4accb5d64aa607f1fcb26a11b5303878451b4fe"
SRC_URI[openssl-0.10.82.sha256sum] = "4de8111e94a56adca7ebfe6c365b6d42f99deb69b74b6becacd4204e2f193aa2"
SRC_URI[openssl-macros-0.1.1.sha256sum] = "a948666b637a0f465e8564c73e89d4dde00d72d4d473cc972f390fc3dcee7d9c"
SRC_URI[openssl-sys-0.9.118.sha256sum] = "19c413d6ec9ebaf5e1c829d355cb3e9f783778d700f135be0921d52fce3c60ae"
SRC_URI[pin-project-1.2.0.sha256sum] = "a6c2123f4708bbd422630a6dae7c03f678b1ce1f1c99e4c4b3beac69c8174cc9"
SRC_URI[pin-project-internal-1.2.0.sha256sum] = "358938fd6df944c1e44bb328b60fb275f13c804aac47ba26a1165d45a3f87756"
SRC_URI[pin-project-lite-0.2.18.sha256sum] = "c938015719552d665fd0553b90d96cd7ff58e83e4c061ca99e6bb78865a72752"
SRC_URI[pkg-config-0.3.34.sha256sum] = "f6b464fbc74e149a392436b17d523f769e057cb6877f6a5c4618bc6f11800548"
SRC_URI[proc-macro2-1.0.107.sha256sum] = "985e7ec9bb745e6ce6535b544d84d6cd6f7ad8bd711c398938ae983b91a766d9"
SRC_URI[quote-1.0.47.sha256sum] = "1fbf4db142a473a8d80c26bbf18454ed458bf8d26c8219c331daecfdbd079001"
SRC_URI[r-efi-6.0.0.sha256sum] = "f8dcc9c7d52a811697d2151c701e0d08956f92b0e24136cf4cf27b57a6a0d9bf"
SRC_URI[rustversion-1.0.23.sha256sum] = "cf54715a573b99ac80df0bc206da022bcd442c974952c7b9720069370852e21f"
SRC_URI[serde-1.0.229.sha256sum] = "4148590afebada386688f18773da617792bf2ef03ffc1e4cbd2b1d45b023e0ba"
SRC_URI[serde_core-1.0.229.sha256sum] = "67dca2c9c51e58a4791a4b1ed58308b39c64224d349a935ab5039aa360942a48"
SRC_URI[serde_derive-1.0.229.sha256sum] = "e7a5d71263a5a7d47b41f6b3f06ba276f10cc18b0931f1799f710578e2309348"
SRC_URI[serde_json-1.0.154.sha256sum] = "e7e9cc8b1b85264074fbcc02a88680c4096b1e47df8f739dceb03bf482f04bd6"
SRC_URI[shlex-2.0.1.sha256sum] = "f8fadd59c855ef2080decdef8ff161eb6661b86933c9d82e5ba29dc602a55aba"
SRC_URI[signal-hook-registry-1.4.8.sha256sum] = "c4db69cba1110affc0e9f7bcd48bbf87b3f4fc7c61fc9155afd4c469eb3d6c1b"
SRC_URI[slab-0.4.12.sha256sum] = "0c790de23124f9ab44544d7ac05d60440adc586479ce501c1d6d7da3cd8c9cf5"
SRC_URI[socket2-0.6.5.sha256sum] = "c3d1e2c7f27f8d4cb10542a02c49005dbd6e93095799d6f3be745fae9f8fedd4"
SRC_URI[strsim-0.11.1.sha256sum] = "7da8b5736845d9f2fcb837ea5d9e2628564b3b043a70948a3f0b778838c5fb4f"
SRC_URI[strum-0.26.3.sha256sum] = "8fec0f0aef304996cf250b31b5a10dee7980c85da9d759361292b8bca5a18f06"
SRC_URI[strum_macros-0.26.4.sha256sum] = "4c6bee85a5a24955dc440386795aa378cd9cf82acd5f764469152d2270e581be"
SRC_URI[syn-2.0.119.sha256sum] = "872831b642d1a07999a962a351ed35b955ea2cfc8f3862091e2a240a84f17297"
SRC_URI[syn-3.0.10.sha256sum] = "4ae9ccbea3a7b0b09443036490c8f34a8b1b760fd345e5c0dae0aac718faea79"
SRC_URI[synstructure-0.13.2.sha256sum] = "728a70f3dbaf5bab7f0c4b1ac8d7ae5ea60a4b5549c8a5914361c99147a709d2"
SRC_URI[tokio-1.54.1.sha256sum] = "b52fd3255258d26fac7264ab5a76b53c13367136dc011d3d6ad4f8ed070396a3"
SRC_URI[tokio-macros-2.7.2.sha256sum] = "78773a2a397f451582ce068015985c33193cf6dea8b74d2a639fe457b2f07b0e"
SRC_URI[tokio-stream-0.1.19.sha256sum] = "a3d06f0b082ba57c26b79407372e57cf2a1e28124f78e9479fe80322cf53420b"
SRC_URI[unicode-ident-1.0.27.sha256sum] = "a2c754d6c33795a1c324727428e5a7dedb5b06195f9890bdbcba760d3e246563"
SRC_URI[utf8parse-0.2.2.sha256sum] = "06abde3611657adf66d383f00b093d7faecc7fa57071cce2578660c9f1010821"
SRC_URI[uuid-1.29.3.sha256sum] = "3c14b15dffa2f4bd712103e857140aa04a4f2ff6e955c77d7eb8e53ee02a3b64"
SRC_URI[vcpkg-0.2.15.sha256sum] = "accd4ea62f7bb7a82fe23066fb0957d48ef677f6eeb8215f372f52e48bb32426"
SRC_URI[wasi-0.11.1+wasi-snapshot-preview1.sha256sum] = "ccf3ec651a847eb01de73ccad15eb7d99f80485de043efb2f370cd654f4ea44b"
SRC_URI[wasm-bindgen-0.2.130.sha256sum] = "a0181c14ddd6c7b8b0b0132a5f1ea2f32be0d893cd3ea97b6e3b28886b8a3d1c"
SRC_URI[wasm-bindgen-macro-0.2.130.sha256sum] = "6e931a02a39abdbf66966fc35902b5ed58344750a70885bed4ebb3d8eaf28ed9"
SRC_URI[wasm-bindgen-macro-support-0.2.130.sha256sum] = "2ca4b54280e455bd831e33a6b8e734baf432fe63fbc35c42ac14ce67a043a53a"
SRC_URI[wasm-bindgen-shared-0.2.130.sha256sum] = "3b1f9c3798725016c6ee5f2f57869ec0831e8bbc0ab0310a86e787e97f41389e"
SRC_URI[windows-link-0.2.1.sha256sum] = "f0805222e57f7521d6a62e36fa9163bc891acd422f971defe97d64e70d0a4fe5"
SRC_URI[windows-sys-0.61.2.sha256sum] = "ae137229bcbd6cdf0f7b80a31df61766145077ddf49416a728b02cb3921ff3fc"
SRC_URI[zeroize-1.9.2.sha256sum] = "85ccff8a4805aba1eaa4cd4afcd49f7557dd2cf5fd79ffc37f8959466effc5aa"
SRC_URI[zmij-1.0.23.sha256sum] = "29666d0abbfad1e3dc4dcf6144730dd3a3ab225bbbdac83319345b1b44ccfc1b"

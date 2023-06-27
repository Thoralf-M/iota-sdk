{
    "targets": [
        {
            "target_name": "index",
            'defines': [
                "NAPI_VERSION=<(napi_build_version)",
            ],
            "win_delay_load_hook": "true",
            "conditions": [
                ["OS == 'mac'", {
                    "cflags+": ["-fvisibility=hidden"],
                    "xcode_settings": {
                        # -fvisibility=hidden
                        "GCC_SYMBOLS_PRIVATE_EXTERN": "YES",

                        # Set minimum target version because we're building on newer
                        # Same as https://github.com/nodejs/node/blob/v11.0.0/common.gypi#L464
                        "MACOSX_DEPLOYMENT_TARGET": "10.7",

                        # Build universal binary to support M1 (Apple silicon)
                        "OTHER_CFLAGS": [
                            "-arch x86_64",
                            "-arch arm64"
                        ],
                        "OTHER_LDFLAGS": [
                            "-arch x86_64",
                            "-arch arm64"
                        ]
                    }
                }]
            ]
        }
    ]
}

"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.ENV = exports.ok = void 0;
const child_process_1 = require("child_process");
const path = require("path");
const fs = require("fs");
const util_1 = require("util");
function ok() {
    let dir_path = process.cwd();
    let src_path = path.join(dir_path, 'src');
    if (fs.existsSync(src_path)) {
        dir_path = src_path;
    }
    const child = (0, child_process_1.spawn)('grep', ['--include', '\*.{js,ts}', '-rE', '(^|[^\\w])ENV\\.(opt\\.|num\\.)?[a-zA-Z0-9_]+', dir_path]);
    child.stdout.on('data', data => {
        let lines = data.toString().split('\n');
        for (const line of lines) {
            if (!line)
                return;
            let match = /ENV\.(opt\.|num\.)?([a-zA-Z0-9_]+)/.exec(line);
            if (!match) {
                console.log('Could not parse ENV grep result:', line.substr(0, 100));
                return;
            }
            let [_, mod, env_var] = match;
            if (mod === 'opt.')
                return;
            let value = process.env[env_var];
            if (mod === 'num.') {
                if (!value || isNaN(parseInt(value))) {
                    console.log(`ENV found required field num.${env_var}, but given value ${(0, util_1.inspect)(value)} is incompatible.`);
                    process.exit(1);
                }
            }
            else {
                if (!value) {
                    console.log(`ENV found required field ${env_var}, but given value ${(0, util_1.inspect)(value)} is incompatible.`);
                    process.exit(1);
                }
            }
        }
    });
    child.stderr.on('data', data => console.log('Error while grepping for ENV variables:', data.toString()));
    // child.on('close', code => {})
}
exports.ok = ok;
exports.ENV = new Proxy({
    opt: new Proxy({}, {
        get(target, name, receiver) {
            return process.env[name];
        },
        set(target, name, value, receiver) {
            throw new Error('Cannot set env value.');
        }
    }),
    num: new Proxy({}, {
        get(target, name, receiver) {
            let v = process.env[name];
            if (v === undefined)
                throw new Error(`Environment variable ${name} is undefined.`);
            return parseInt(v);
        },
        set(target, name, value, receiver) {
            throw new Error('Cannot set env value.');
        }
    }),
    ok
}, {
    get(target, name, receiver) {
        if (['num', 'opt', 'ok'].includes(name)) {
            return Reflect.get(target, name, receiver);
        }
        let v = process.env[name];
        if (v === undefined)
            throw new Error(`Environment variable ${name} is undefined.`);
        return v;
    },
    set(target, name, value, receiver) {
        throw new Error('Cannot set env value.');
    }
});
//# sourceMappingURL=index.js.map
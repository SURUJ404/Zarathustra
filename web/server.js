const express = require('express');
const cors = require('cors');
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const app = express();
const PORT = process.env.PORT || 4000;
const HOST = process.env.HOST || '0.0.0.0';
const IS_VERCEL = process.env.VERCEL === '1';

const API_DIR = __dirname;
const PUBLIC_DIR = path.join(API_DIR, 'public');
const WORK_DIR = process.env.WORK_DIR || (IS_VERCEL ? '/tmp/astra-work' : path.join(API_DIR, 'work'));
const BIN_PATH = process.env.ASTRA_BIN || (IS_VERCEL ? path.join(API_DIR, '..', 'api') : path.join(API_DIR, '..', 'target', 'debug'));
const MAX_FILE_SIZE = parseInt(process.env.MAX_FILE_SIZE || '50', 10);

fs.mkdirSync(WORK_DIR, { recursive: true });

app.use(cors());
app.use(express.json({ limit: `${MAX_FILE_SIZE}mb` }));
app.use(express.static(PUBLIC_DIR));

app.get('/api/health', (req, res) => {
    const bin = path.join(BIN_PATH, 'astra');
    res.json({
        status: 'ok',
        timestamp: new Date().toISOString(),
        astra: fs.existsSync(bin),
        workDir: WORK_DIR,
    });
});

function sanitize(vals) {
    if (!Array.isArray(vals)) return [];
    return vals.filter(v => typeof v === 'string' && /^[a-zA-Z0-9_,.\-]+$/.test(v));
}

function cleanup(dir) {
    try { fs.rmSync(dir, { recursive: true, force: true }); } catch {}
}

function run(input, cmd, privateVals, publicVals, reuseDir) {
    const bin = path.join(BIN_PATH, 'astra');
    if (!fs.existsSync(bin)) {
        return { ok: false, error: 'astra binary not found' };
    }
    try { fs.chmodSync(bin, 0o755); } catch (e) {}
    const reqDir = reuseDir || path.join(WORK_DIR, crypto.randomUUID());
    fs.mkdirSync(reqDir, { recursive: true });
    const f = path.join(reqDir, 'zara_main.zara');
    try {
        fs.writeFileSync(f, input);
        const pv = sanitize(privateVals);
        const pb = sanitize(publicVals);
        const privateArg = pv.length ? `-r ${pv.join(',')}` : '';
        const publicArg = pb.length ? `-p ${pb.join(',')}` : '';
        const out = execSync(`"${bin}" ${cmd} ${privateArg} ${publicArg} "${f}"`, {
            cwd: reqDir,
            encoding: 'utf-8',
            maxBuffer: MAX_FILE_SIZE * 1024 * 1024,
            timeout: 60000,
            shell: false,
        });
        return { ok: true, stdout: out.trim(), dir: reqDir };
    } catch (e) {
        return {
            ok: false,
            error: e.stderr?.trim() || e.message,
            stdout: e.stdout?.trim() || '',
            dir: reqDir,
        };
    }
}

function readJsonOrNull(file) {
    try {
        return JSON.parse(fs.readFileSync(file, 'utf-8'));
    } catch {
        return null;
    }
}

app.post('/api/compile', (req, res) => {
    if (typeof req.body.code !== 'string') {
        return res.status(400).json({ ok: false, error: 'code must be a string' });
    }
    const r = run(req.body.code, 'prove compile', req.body.private, req.body.public);
    cleanup(r.dir);
    delete r.dir;
    res.json(r);
});

app.post('/api/prove', (req, res) => {
    if (typeof req.body.code !== 'string') {
        return res.status(400).json({ ok: false, error: 'code must be a string' });
    }
    const r = run(req.body.code, 'prove prove', req.body.private, req.body.public);
    let proof = null;
    if (r.ok) {
        proof = readJsonOrNull(path.join(r.dir, 'proof.json'));
    }
    cleanup(r.dir);
    delete r.dir;
    res.json({ ...r, proof });
});

app.post('/api/verify', (req, res) => {
    if (typeof req.body.code !== 'string') {
        return res.status(400).json({ ok: false, error: 'code must be a string' });
    }
    const r = run(req.body.code, 'prove prove', req.body.private, req.body.public);
    if (!r.ok) {
        cleanup(r.dir);
        delete r.dir;
        return res.json(r);
    }
    const v = run('', 'prove verify', req.body.private, req.body.public, r.dir);
    cleanup(r.dir);
    delete r.dir;
    delete v.dir;
    res.json({ ok: v.ok, stdout: v.stdout, error: v.error });
});

if (!IS_VERCEL) {
    app.listen(PORT, HOST, () => {
        console.log(`Astra Web IDE running at http://${HOST}:${PORT}`);
    });
}

module.exports = app;

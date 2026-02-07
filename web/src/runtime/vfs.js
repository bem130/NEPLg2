export class VFS {
    constructor() {
        this.files = new Map();
        // Populated by main.js
    }

    writeFile(path, content) {
        if (!path.startsWith('/')) path = '/' + path;
        this.files.set(path, content);
    }

    readFile(path) {
        if (!path.startsWith('/')) path = '/' + path;
        if (!this.files.has(path)) {
            throw new Error(`File not found: ${path}`);
        }
        return this.files.get(path);
    }

    exists(path) {
        if (!path.startsWith('/')) path = '/' + path;
        return this.files.has(path);
    }

    isDir(path) {
        if (!path.startsWith('/')) path = '/' + path;
        if (path === '/') return true;
        const prefix = path.endsWith('/') ? path : path + '/';
        for (const key of this.files.keys()) {
            if (key.startsWith(prefix)) return true;
        }
        return false;
    }

    listDir(dirPath) {
        if (!dirPath.startsWith('/')) dirPath = '/' + dirPath;
        if (!dirPath.endsWith('/')) dirPath += '/';

        const results = new Set();
        for (const path of this.files.keys()) {
            if (path.startsWith(dirPath)) {
                const relative = path.substring(dirPath.length);
                const firstSegment = relative.split('/')[0];
                if (firstSegment) {
                    results.add(firstSegment);
                }
            }
        }
        return Array.from(results).sort();
    }

    deleteFile(path) {
        if (!path.startsWith('/')) path = '/' + path;
        return this.files.delete(path);
    }
}

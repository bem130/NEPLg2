// @ts-nocheck
class MarkdownLanguageProvider extends BaseLanguageProvider {
    constructor() {
        super('dist_ts/language/markdown/md-worker.js');
    }
}

window.MarkdownLanguageProvider = MarkdownLanguageProvider;

// @ts-nocheck
class JavaScriptLanguageProvider extends BaseLanguageProvider {
    constructor() {
        super('dist_ts/language/javascript/js-worker.js');
    }
}

window.JavaScriptLanguageProvider = JavaScriptLanguageProvider;

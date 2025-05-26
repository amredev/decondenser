import * as assert from "assert";

// You can import and use all API from the 'vscode' module
// as well as import your extension to test it
import * as vscode from "vscode";
// import * as myExtension from '../../extension';

import * as decondenser from "../extension";

function formatUnescapedText(text: string): string {
    const uglyText = decondenser.unescapeText(text);

    const prettyText = decondenser.formatUglyText(uglyText, "    ");

    return prettyText;
}

suite("Extension Test Suite", () => {
    vscode.window.showInformationMessage("Start all tests.");

    test("Smoke test", () => {
        const text = `Test {key1:"value1","key2": "value2"}[[1],[2],[3],]{"key1":"value1","key2":[123]}`;
        const expected = `\
Test {
    key1:"value1",
    "key2": "value2"
}[
    [
        1
    ],
    [
        2
    ],
    [
        3
    ]
]{
    "key1":"value1",
    "key2":[
        123
    ]
}`;
        const output = decondenser.formatUglyText(text, "    ");
        assert.strictEqual(expected, output);
    });

    test("Escape test", () => {
        const text = `{"key": "\\n\\r\\t"}`;
        const output = decondenser.formatUglyText(text, "    ");
        assert.strictEqual(`{\n    "key": "\\n\\r\\t"\n}`, output);
    });

    test("Unescape test \\n", () => {
        const text = `{"key": "val\nue"}`;
        const output = formatUnescapedText(text);
        assert.strictEqual(`{\n    "key": "val\nue"\n}`, output);
    });

    test("Unescape test \\r", () => {
        const text = `{"key": "val\rue"}`;
        const output = formatUnescapedText(text);
        assert.strictEqual(`{\n    "key": "val\rue"\n}`, output);
    });

    test("Unescape test \\t", () => {
        const text = `{"key": "val\tue"}`;
        const output = formatUnescapedText(text);
        assert.strictEqual(`{\n    "key": "val\tue"\n}`, output);
    });
});

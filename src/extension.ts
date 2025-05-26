// The module 'vscode' contains the VS Code extensibility API
// Import the module and reference it with the alias vscode in your code below
import * as vscode from "vscode";

// This method is called when your extension is activated
// Your extension is activated the very first time the command is executed
export function activate(context: vscode.ExtensionContext) {
    function addCommand(command: string, action: () => void) {
        const registeredCommand = vscode.commands.registerCommand(
            command,
            action,
        );
        context.subscriptions.push(registeredCommand);
    }

    addCommand("decondenser.bracketsPrettify", () => {
        bracketsPrettify(false);
    });

    addCommand("decondenser.bracketsPrettify.unescape", () => {
        bracketsPrettify(true);
    });
}

function bracketsPrettify(unescape: boolean) {
    const editor = vscode.window.activeTextEditor;
    if (!editor) {
        return;
    }

    let selection;
    if (!editor.selection || editor.selection.isEmpty) {
        const firstLine = editor.document.lineAt(0);
        const lastLine = editor.document.lineAt(editor.document.lineCount - 1);
        selection = new vscode.Selection(
            firstLine.range.start,
            lastLine.range.end,
        );
    } else {
        selection = editor.selection;
    }
    const uglyText = editor.document.getText(selection);

    if (uglyText.length === 0) {
        return;
    }

    let text;
    if (unescape) {
        text = unescapeText(uglyText);
    } else {
        text = uglyText;
    }

    const indentationSize = vscode.workspace
        .getConfiguration("decondenser")
        .get("indentationSize");
    let prettyText;
    if (typeof indentationSize === "number") {
        const indentation = " ".repeat(indentationSize);
        prettyText = formatUglyText(text, indentation);
    } else {
        prettyText = formatUglyText(text, "    ");
    }

    editor.edit((editBuilder) => {
        editBuilder.replace(selection, prettyText);
    });
}

export function unescapeText(input: string): string {
    let output: string[] = [];
    let backslash = false;

    for (const char of input) {
        if (backslash) {
            switch (char) {
                case "n": {
                    output.push("\n");
                    break;
                }
                case "r": {
                    output.push("\r");
                    break;
                }
                case "t": {
                    output.push("\t");
                    break;
                }
                default: {
                    output.push(char);
                }
            }
            backslash = false;

            continue;
        }

        if (char === "\\") {
            backslash = true;
            continue;
        }

        output.push(char);
    }

    return output.join("");
}

enum State {
    // This state is assigned:
    // - at the start of parsing
    // - when met \ after comma: key: value,\
    // - when met " end of the string
    // - when met open bracket after comma: [[1],[2]]
    // - when met closing bracket after comma (no trailing comma added): [[1,2,]]
    // - when met any character after comma: {key1: value1,key2: [1,2,3]}
    Start,
    // This state is assigned:
    // - when met " start of the string: {key: "abc"}
    // - when met \ is escape state: "\\abc"
    // - when met " after comma: ["value1","value2", "value3"]
    // - when met any character in escape state: "\X"
    String,
    // This state is assigned:
    // - when met \ in string: "\"
    Escape,
    // This state is assigned:
    // - when met , in start state: [1,2,3,] | {key: value,}
    AfterComma,
}

export function formatUglyText(input: string, indentation: string): string {
    let state: State = (() => {
        return State.Start;
    })();
    let output: string[] = [];
    let indentLevel = 0;

    function pushAfterCommaAndIndent(character: string): string[] {
        return output.concat([
            ",\n",
            createIndentation(indentLevel),
            character,
        ]);
    }

    function createIndentation(indentLevel: number): string {
        return indentation.repeat(Math.abs(indentLevel));
    }

    for (const char of input) {
        switch (char) {
            case "\\": {
                switch (state) {
                    case State.Start: {
                        output.push("\\");
                        break;
                    }
                    case State.String: {
                        // met in string \ character, start escaping
                        state = State.Escape;
                        output.push("\\");
                        break;
                    }
                    case State.Escape: {
                        // in string, after first \, met second \ character
                        state = State.String;
                        output.push("\\");
                        break;
                    }
                    case State.AfterComma: {
                        // met \ after comma
                        state = State.Start;
                        output = pushAfterCommaAndIndent("\\");
                        break;
                    }
                    default:
                        state satisfies never;
                }

                break;
            }

            case '"': {
                switch (state) {
                    case State.Start: {
                        // start of the string
                        state = State.String;
                        output.push('"');
                        break;
                    }
                    case State.String: {
                        // end of the string
                        state = State.Start;
                        output.push('"');
                        break;
                    }
                    case State.Escape: {
                        // escaped " in string
                        state = State.String;
                        output.push('"');
                        break;
                    }
                    case State.AfterComma: {
                        // met " after comma
                        state = State.String;
                        output = pushAfterCommaAndIndent('"');
                        break;
                    }
                    default:
                        state satisfies never;
                }

                break;
            }

            case " ": {
                switch (state) {
                    case State.Start: {
                        output.push(" ");
                        break;
                    }
                    case State.String: {
                        output.push(" ");
                        break;
                    }
                    case State.Escape: {
                        state = State.String;
                        output.push(" ");
                        break;
                    }
                    case State.AfterComma: {
                        break;
                    }
                    default:
                        state satisfies never;
                }

                break;
            }

            case ",": {
                switch (state) {
                    case State.Start: {
                        state = State.AfterComma;
                        break;
                    }
                    case State.String: {
                        output.push(",");
                        break;
                    }
                    case State.Escape: {
                        state = State.String;
                        output.push(",");
                        break;
                    }
                    case State.AfterComma: {
                        output.push(",");
                        break;
                    }
                    default:
                        state satisfies never;
                }

                break;
            }

            case "(":
            case "[":
            case "{": {
                switch (state) {
                    case State.Start: {
                        indentLevel = indentLevel + 1;
                        output = output.concat([
                            char,
                            "\n",
                            createIndentation(indentLevel),
                        ]);
                        break;
                    }
                    case State.String: {
                        output.push(char);
                        break;
                    }
                    case State.Escape: {
                        state = State.String;
                        output.push(char);
                        break;
                    }
                    case State.AfterComma: {
                        state = State.Start;
                        output = pushAfterCommaAndIndent(char);
                        indentLevel = indentLevel + 1;
                        output = output.concat([
                            "\n",
                            createIndentation(indentLevel),
                        ]);
                        break;
                    }
                    default:
                        state satisfies never;
                }

                break;
            }

            case ")":
            case "]":
            case "}": {
                switch (state) {
                    case State.Start: {
                        indentLevel = indentLevel - 1;
                        output = output.concat([
                            "\n",
                            createIndentation(indentLevel),
                            char,
                        ]);
                        break;
                    }
                    case State.String: {
                        output.push(char);
                        break;
                    }
                    case State.Escape: {
                        state = State.String;
                        output.push(char);
                        break;
                    }
                    case State.AfterComma: {
                        state = State.Start;
                        indentLevel = indentLevel - 1;
                        output = output.concat([
                            "\n",
                            createIndentation(indentLevel),
                            char,
                        ]);
                        break;
                    }
                    default:
                        state satisfies never;
                }

                break;
            }

            default: {
                switch (state) {
                    case State.Start: {
                        output.push(char);
                        break;
                    }
                    case State.String: {
                        output.push(char);
                        break;
                    }
                    case State.Escape: {
                        state = State.String;
                        output.push(char);
                        break;
                    }
                    case State.AfterComma: {
                        state = State.Start;
                        output = pushAfterCommaAndIndent(char);
                        break;
                    }
                    default:
                        state satisfies never;
                }
            }
        }
    }

    return output.join("");
}

// This method is called when your extension is deactivated
export function deactivate() {}

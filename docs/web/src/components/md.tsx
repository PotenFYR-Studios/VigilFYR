import { useState } from "react";

export function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);

  return (
    <button
      type="button"
      className="copy-btn"
      onClick={() => {
        navigator.clipboard?.writeText(text).then(
          () => {
            setCopied(true);
            setTimeout(() => setCopied(false), 1400);
          },
          () => undefined,
        );
      }}
    >
      {copied ? "Copied!" : "Copy"}
    </button>
  );
}

function CodeBlock({ code, language }: { code: string; language?: string }) {
  return (
    <pre data-lang={language ?? ""}>
      <code>{code}</code>
      <CopyButton text={code} />
    </pre>
  );
}

const INLINE_RE = /(`[^`]+`)|(\*\*[^*]+\*\*)|(\*[^*]+\*)|(\[[^\]]+\]\([^)\s]+\))/g;

function inline(text: string, keyBase: string): React.ReactNode[] {
  const parts: React.ReactNode[] = [];
  let last = 0;
  let key = 0;
  let match: RegExpExecArray | null;
  INLINE_RE.lastIndex = 0;

  while ((match = INLINE_RE.exec(text)) !== null) {
    if (match.index > last) parts.push(text.slice(last, match.index));
    const token = match[0];
    if (token.startsWith("`")) {
      parts.push(<code key={`${keyBase}-${key++}`}>{token.slice(1, -1)}</code>);
    } else if (token.startsWith("**")) {
      parts.push(<strong key={`${keyBase}-${key++}`}>{token.slice(2, -2)}</strong>);
    } else if (token.startsWith("*")) {
      parts.push(<em key={`${keyBase}-${key++}`}>{token.slice(1, -1)}</em>);
    } else {
      const parsed = /\[([^\]]+)\]\(([^)\s]+)\)/.exec(token)!;
      const external = /^https?:\/\//.test(parsed[2]);
      parts.push(
        <a
          key={`${keyBase}-${key++}`}
          href={parsed[2]}
          target={external ? "_blank" : undefined}
          rel={external ? "noopener noreferrer" : undefined}
        >
          {parsed[1]}
        </a>,
      );
    }
    last = match.index + token.length;
  }
  if (last < text.length) parts.push(text.slice(last));
  return parts;
}

export function Markdown({ content }: { content: string }) {
  const lines = content.split("\n");
  const nodes: React.ReactNode[] = [];
  let index = 0;
  let key = 0;

  while (index < lines.length) {
    const line = lines[index];

    if (line.startsWith("```")) {
      const language = line.slice(3).trim() || undefined;
      const code: string[] = [];
      index++;
      while (index < lines.length && !lines[index].startsWith("```")) {
        code.push(lines[index].replace(/^ {3}/, ""));
        index++;
      }
      index++;
      nodes.push(
        <CodeBlock key={`code-${key++}`} code={code.join("\n")} language={language} />,
      );
      continue;
    }

    if (/^ {4}\S/.test(line)) {
      const code: string[] = [];
      while (index < lines.length && (/^ {4}/.test(lines[index]) || lines[index].trim() === "")) {
        if (lines[index].trim() === "" && index + 1 < lines.length && !/^ {4}/.test(lines[index + 1])) {
          break;
        }
        code.push(lines[index].replace(/^ {4}/, ""));
        index++;
      }
      nodes.push(
        <CodeBlock
          key={`indent-${key++}`}
          code={code.join("\n").replace(/\n+$/, "")}
        />,
      );
      continue;
    }

    if (
      line.includes(" | ") &&
      index + 1 < lines.length &&
      /^\s*\|?[\s:-]*-[\s|:-]*$/.test(lines[index + 1]) &&
      lines[index + 1].includes("-")
    ) {
      const cells = (row: string) =>
        row
          .replace(/^\s*\|/, "")
          .replace(/\|\s*$/, "")
          .split("|")
          .map((cell) => cell.trim());
      const head = cells(line);
      index += 2;
      const rows: string[][] = [];
      while (index < lines.length && lines[index].includes("|")) {
        rows.push(cells(lines[index]));
        index++;
      }
      nodes.push(
        <div key={`table-${key++}`} className="table-scroll">
          <table className="doc-table">
            <thead>
              <tr>
                {head.map((cell, cellIndex) => (
                  <th key={cellIndex}>{inline(cell, `h-${key}-${cellIndex}`)}</th>
                ))}
              </tr>
            </thead>
            <tbody>
              {rows.map((row, rowIndex) => (
                <tr key={rowIndex}>
                  {row.map((cell, cellIndex) => (
                    <td key={cellIndex}>{inline(cell, `r-${key}-${rowIndex}-${cellIndex}`)}</td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>,
      );
      continue;
    }

    if (line.startsWith("#### ")) {
      const text = line.slice(5).trim();
      nodes.push(
        <h4 key={`h4-${key++}`} id={slugify(text)}>
          {inline(text, `h4-${key}`)}
        </h4>,
      );
    } else if (line.startsWith("### ")) {
      const text = line.slice(4).trim();
      nodes.push(
        <h3 key={`h3-${key++}`} id={slugify(text)}>
          {inline(text, `h3-${key}`)}
        </h3>,
      );
    } else if (line.startsWith("## ")) {
      const text = line.slice(3).trim();
      nodes.push(
        <h2 key={`h2-${key++}`} id={slugify(text)}>
          {inline(text, `h2-${key}`)}
        </h2>,
      );
    } else if (line.startsWith("# ")) {
      nodes.push(
        <h1 key={`h1-${key++}`}>{inline(line.slice(2).trim(), `h1-${key}`)}</h1>,
      );
    } else if (/^[-*] /.test(line)) {
      const items: string[] = [];
      while (index < lines.length && /^[-*] /.test(lines[index])) {
        items.push(lines[index].slice(2));
        index++;
      }
      nodes.push(
        <ul key={`ul-${key++}`}>
          {items.map((item, itemIndex) => (
            <li key={itemIndex}>{inline(item, `li-${key}-${itemIndex}`)}</li>
          ))}
        </ul>,
      );
      continue;
    } else if (/^\d+\. /.test(line)) {
      const items: string[] = [];
      while (index < lines.length && /^\d+\. /.test(lines[index])) {
        items.push(lines[index].replace(/^\d+\. /, ""));
        index++;
      }
      nodes.push(
        <ol key={`ol-${key++}`}>
          {items.map((item, itemIndex) => (
            <li key={itemIndex}>{inline(item, `oli-${key}-${itemIndex}`)}</li>
          ))}
        </ol>,
      );
      continue;
    } else if (line.startsWith("> ")) {
      nodes.push(
        <blockquote key={`bq-${key++}`}>{inline(line.slice(2), `bq-${key}`)}</blockquote>,
      );
    } else if (line.trim() === "---") {
      nodes.push(<hr key={`hr-${key++}`} />);
    } else if (line.trim() !== "") {
      nodes.push(<p key={`p-${key++}`}>{inline(line, `p-${key}`)}</p>);
    }

    index++;
  }

  return <div className="md-body">{nodes}</div>;
}

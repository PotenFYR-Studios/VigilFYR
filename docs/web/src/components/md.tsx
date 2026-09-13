export function Markdown({ content }: { content: string }) {
  const lines = content.split("\n");
  const nodes: React.ReactNode[] = [];
  let paragraph: string[] = [];
  let list: string[] = [];
  let inCode = false;
  let code: string[] = [];

  const inline = (text: string) =>
    text
      .split(/(`[^`]+`|\*\*[^*]+\*\*|\[[^\]]+\]\([^)]+\))/g)
      .filter(Boolean)
      .map((part, index) => {
        if (part.startsWith("`")) return <code key={index}>{part.slice(1, -1)}</code>;
        if (part.startsWith("**")) return <strong key={index}>{part.slice(2, -2)}</strong>;
        const match = /^\[([^\]]+)\]\(([^)]+)\)$/.exec(part);
        if (match) {
          return (
            <a key={index} href={match[2]}>
              {match[1]}
            </a>
          );
        }
        return <span key={index}>{part}</span>;
      });

  const flushParagraph = () => {
    if (paragraph.length) {
      nodes.push(<p key={`p-${nodes.length}`}>{inline(paragraph.join(" "))}</p>);
      paragraph = [];
    }
  };

  const flushList = () => {
    if (list.length) {
      nodes.push(
        <ul key={`ul-${nodes.length}`}>
          {list.map((item, index) => (
            <li key={index}>{inline(item)}</li>
          ))}
        </ul>,
      );
      list = [];
    }
  };

  for (const line of lines) {
    if (line.startsWith("```")) {
      if (inCode) {
        nodes.push(<pre key={`pre-${nodes.length}`}><code>{code.join("\n")}</code></pre>);
        code = [];
        inCode = false;
      } else {
        flushParagraph();
        flushList();
        inCode = true;
      }
      continue;
    }
    if (inCode) {
      code.push(line);
      continue;
    }
    if (line.startsWith("# ")) {
      flushParagraph();
      flushList();
      nodes.push(<h1 key={`h1-${nodes.length}`}>{inline(line.slice(2))}</h1>);
    } else if (line.startsWith("## ")) {
      flushParagraph();
      flushList();
      nodes.push(<h2 key={`h2-${nodes.length}`}>{inline(line.slice(3))}</h2>);
    } else if (line.startsWith("### ")) {
      flushParagraph();
      flushList();
      nodes.push(<h3 key={`h3-${nodes.length}`}>{inline(line.slice(4))}</h3>);
    } else if (line.startsWith("- ")) {
      flushParagraph();
      list.push(line.slice(2));
    } else if (!line.trim()) {
      flushParagraph();
      flushList();
    } else {
      flushList();
      paragraph.push(line);
    }
  }
  flushParagraph();
  flushList();
  return <div className="md-body">{nodes}</div>;
}

import { useEffect, useId, useRef, useState } from "react";

export function NumberTicker({ value, className }: { value: number; className?: string }) {
  const [display, setDisplay] = useState(0);
  const previous = useRef(0);

  useEffect(() => {
    const start = performance.now();
    const from = previous.current;
    const step = (time: number) => {
      const progress = Math.min(1, (time - start) / 900);
      const eased = 1 - (1 - progress) ** 3;
      setDisplay(Math.round(from + (value - from) * eased));
      if (progress < 1) requestAnimationFrame(step);
      else previous.current = value;
    };
    const frame = requestAnimationFrame(step);
    return () => cancelAnimationFrame(frame);
  }, [value]);

  return (
    <span className={className} aria-label={String(value)}>
      {display}
    </span>
  );
}

export function MagicCard({
  children,
  className = "",
}: {
  children: React.ReactNode;
  className?: string;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState({ x: -400, y: -400 });
  const [inside, setInside] = useState(false);

  return (
    <div
      ref={ref}
      className={`relative overflow-hidden rounded-2xl border border-line-light bg-white/[0.02] transition-colors duration-300 hover:border-brand-sky/50 ${className}`}
      onMouseMove={(event) => {
        const rect = ref.current?.getBoundingClientRect();
        if (rect) {
          setPosition({ x: event.clientX - rect.left, y: event.clientY - rect.top });
        }
      }}
      onMouseEnter={() => setInside(true)}
      onMouseLeave={() => setInside(false)}
    >
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 opacity-0 transition-opacity duration-300"
        style={{
          opacity: inside ? 1 : 0,
          background: `radial-gradient(340px circle at ${position.x}px ${position.y}px, rgba(14,165,233,0.13), transparent 70%)`,
        }}
      />
      {children}
    </div>
  );
}

export function DotPattern({ className = "" }: { className?: string }) {
  const id = useId();
  return (
    <svg aria-hidden className={`pointer-events-none absolute inset-0 h-full w-full ${className}`}>
      <defs>
        <pattern id={id} width="28" height="28" patternUnits="userSpaceOnUse">
          <circle cx="2" cy="2" r="1.2" fill="rgba(14,165,233,0.18)" />
        </pattern>
      </defs>
      <rect width="100%" height="100%" fill={`url(#${id})`} />
    </svg>
  );
}

export function GlowOrb({ className = "" }: { className?: string }) {
  return <div aria-hidden className={`orb ${className}`} />;
}

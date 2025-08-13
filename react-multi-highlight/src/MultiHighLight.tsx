import React, { useMemo, useState } from "react";

type Selection = {
    start: number; // inclusive
    end: number;   // exclusive
    color: string; // CSS color
    label?: string; // optional label for tooltips (e.g., "brown fox")
};

type MultiHighlightProps = {
    text: string;
    selections: Selection[];
    className?: string;
    stripeWidthPx?: number; // width of each diagonal stripe band
};

export const MultiHighlight: React.FC<MultiHighlightProps> = ({
    text,
    selections,
    className,
    stripeWidthPx = 6,
}) => {
    if (!text) return null;

    // 1) Build split boundaries
    const points = useMemo(() => {
        const b = new Set<number>([0, text.length]);
        for (const { start, end } of selections) {
            const s = Math.max(0, Math.min(text.length, start));
            const e = Math.max(0, Math.min(text.length, end));
            if (s < e) {
                b.add(s);
                b.add(e);
            }
        }
        return Array.from(b).sort((a, c) => a - c);
    }, [text, selections]);

    // 2) Precompute helper: does selection cover segment?
    const segments = useMemo(() => {
        const segs: {
            start: number;
            end: number;
            text: string;
            covering: Selection[];
        }[] = [];
        for (let i = 0; i < points.length - 1; i++) {
            const segStart = points[i];
            const segEnd = points[i + 1];
            const segText = text.slice(segStart, segEnd);
            const covering = selections.filter(
                ({ start, end }) => start < segEnd && end > segStart
            );
            segs.push({ start: segStart, end: segEnd, text: segText, covering });
        }
        return segs;
    }, [points, selections, text]);

    return (
        <span className={className} style={{ lineHeight: 1.8 }}>
            {segments.map((seg) => (
                <Segment
                    key={`${seg.start}-${seg.end}`}
                    text={seg.text}
                    covering={seg.covering}
                    stripeWidthPx={stripeWidthPx}
                />
            ))}
        </span>
    );
};

// Renders one segment with (optional) highlight + tooltip on hover
const Segment: React.FC<{
    text: string;
    covering: Selection[];
    stripeWidthPx: number;
}> = ({ text, covering, stripeWidthPx }) => {
    const [hovered, setHovered] = useState(false);

    const style = useMemo<React.CSSProperties | undefined>(() => {
        if (covering.length === 0) return undefined;
        if (covering.length === 1) {
            return {
                backgroundColor: covering[0].color,
                borderRadius: 3,
                padding: "0 2px",
            };
        }
        // Multiple selections → 45° diagonal opaque stripes
        // Build repeating-linear-gradient with equal-sized bands per color.
        const band = Math.max(2, Math.floor(stripeWidthPx));
        // Create a cycle of color stops: c1 0..band, c2 band..2band, c3 2band..3band, then repeat
        const stops = covering
            .map((sel, i) => {
                const from = i * band;
                const to = (i + 1) * band;
                return `${sel.color} ${from}px ${to}px`;
            })
            .join(", ");

        return {
            backgroundImage: `repeating-linear-gradient(45deg, ${stops})`,
            borderRadius: 3,
            padding: "0 2px",
            boxShadow: "inset 0 0 0 1px rgba(0,0,0,0.06)",
        };
    }, [covering, stripeWidthPx]);

    const tooltipContent = useMemo(() => {
        if (covering.length === 0) return null;
        return (
            <div
                style={{
                    display: "flex",
                    flexDirection: "column",
                    gap: 4,
                    fontSize: 12,
                    maxWidth: 260,
                }}
            >
                {covering.map((sel, idx) => (
                    <div key={idx} style={{ display: "flex", alignItems: "center", gap: 8 }}>
                        <span
                            aria-hidden
                            style={{
                                width: 10,
                                height: 10,
                                borderRadius: 2,
                                background: sel.color,
                                boxShadow: "inset 0 0 0 1px rgba(0,0,0,.2)",
                                flex: "0 0 auto",
                            }}
                        />
                        <span style={{ lineHeight: 1.2 }}>
                            {sel.label ?? "Selection"}
                            {!!sel.label && (
                                <span style={{ opacity: 0.7 }}> ({sel.start}–{sel.end})</span>
                            )}
                            {!sel.label && (
                                <span style={{ opacity: 0.7 }}> [{sel.start}–{sel.end}]</span>
                            )}
                        </span>
                    </div>
                ))}
            </div>
        );
    }, [covering]);

    return (
        <span
            onMouseEnter={() => setHovered(true)}
            onMouseLeave={() => setHovered(false)}
            style={{
                position: "relative",
                whiteSpace: "pre-wrap", // preserve spaces/newlines
                ...style,
            }}
        >
            {text}
            {/* Tooltip (rendered above) */}
            {hovered && covering.length > 0 && (
                <div
                    role="tooltip"
                    style={{
                        position: "absolute",
                        left: "50%",
                        bottom: "100%",
                        transform: "translate(-50%, -8px)",
                        background: "rgba(20,20,20,0.95)",
                        color: "white",
                        padding: "8px 10px",
                        borderRadius: 8,
                        whiteSpace: "normal",
                        pointerEvents: "none",
                        zIndex: 1000,
                        boxShadow:
                            "0 8px 24px rgba(0,0,0,.25), 0 1px 2px rgba(0,0,0,.25) inset",
                    }}
                >
                    {tooltipContent}
                    {/* little arrow */}
                    <span
                        aria-hidden
                        style={{
                            position: "absolute",
                            left: "50%",
                            top: "100%",
                            transform: "translateX(-50%)",
                            width: 0,
                            height: 0,
                            borderLeft: "7px solid transparent",
                            borderRight: "7px solid transparent",
                            borderTop: "7px solid rgba(20,20,20,0.95)",
                            pointerEvents: "none",
                        }}
                    />
                </div>
            )}
        </span>
    );
};

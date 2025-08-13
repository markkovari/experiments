import { MultiHighlight } from "./MultiHighLight";

const text = "The quick brown fox jumps over the lazy dog";

// "brown fox"
const sel1 = { start: 10, end: 19, color: "#ffea8a" };
// "fox jump" (note: selection is "fox jump" which overlaps "brown fox" on "fox")
const sel2 = { start: 16, end: 24, color: "#a0e7ff" };

const sel3 = { start: 16, end: 24, color: "#5fad34ff" };

export function Demo() {
    return (
        <div style={{ lineHeight: 1.8, fontFamily: "system-ui, sans-serif" }}>
            <MultiHighlight text={text} selections={[sel1, sel2, sel3]} stripeWidthPx={6} />
        </div>
    );
}
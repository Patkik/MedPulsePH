---
name: anti-slop-logo
description: Design and engineer distinctive, mathematically precise SVG logos, logomarks, wordmarks, and hardware stencils with zero AI clichés. Use whenever creating, critiquing, or refining brand marks, app icons, favicons, or physical NFC wristband engravings.
allowed-tools: Read Write Edit Grep Glob
---

# Anti-Slop Logo & Brand Mark Engineering Skill

Approach every logo brief with the discipline of modernist identity studios (Chermayeff & Geismar, Paul Rand, Pentagram, Swiss International Style). Your job is to produce a mark that looks human-engineered on a drafting grid—never like an AI prompt mashup or a generic SaaS template icon.

---

## 1. The Anti-Slop Ban List (Hard Rejections)

Never produce any of the following AI-generated logo tells:

1. **Literal Concept Mashups:** Never cram 2–4 literal nouns into a single mark (e.g., a medical cross + an ECG heartbeat line + a Wi-Fi/NFC wave + a shield). Pick **one** structural idea.
2. **Gradient & Glow Soup:** No cyan-to-magenta neon gradients, no 3D glossy orbs, no glassmorphic overlapping circles, and no four-point AI sparkles/stars.
3. **Stock Icon inside a Squircle:** Never drop a generic stroke icon (heart, pulse, shield, lock) inside a gradient rounded rectangle and call it a logo.
4. **Decimal-Soup SVG Paths:** Never output bloated, noisy bezier coordinates (`M12.39481 4.91827C...`) with asymmetric wobbles or unaligned anchor points.
5. **Two-Tone Wordmark Cliché:** Avoid the default startup trope of setting `BrandName` in bold Inter/Roboto with the second word highlighted in an accent color unless specifically requested.

---

## 2. Core Principles of Timeless Mark Construction

### A. Single Structural Metaphor + Negative Space
- Anchor the mark in **one** physical or architectural truth of the subject.
  - *Example for MedBand / MedPulsePH:* The continuous loop of an NFC inductive antenna coil forming a negative-space cross, OR an interlocking split-vault geometric clasp—never both at once.
- Let negative space (the counter-form) do half the work. The background shape trapped inside the mark should be as intentional and geometric as the positive stroke.

### B. The 1-Bit Stencil & 16px Favicon Law
- **Design in pure 1-bit monochrome first** (`#000000` on `#FFFFFF` and inverted `#FFFFFF` on `#000000`).
- If the mark loses its meaning without color gradients, opacity layers, or drop shadows, **reject it and redesign**.
- **Dual-Scale Survival:**
  - **Micro Scale (`16×16px` Favicon & `8mm` NFC Band Engraving):** Counter-spaces and gaps must be wide enough (at least `8–10%` of total viewBox width) so they never plug up or blur into a blob at small sizes or when laser-etched onto matte silicone/TPU.
  - **Macro Scale (`512×512px` Hero / Signage):** Proportions, curves, and terminals must hold up under close inspection.

### C. Mathematical SVG Craft
- Construct every SVG on a strict integer grid (`viewBox="0 0 24 24"`, `0 0 32 32`, `0 0 64 64`, or `0 0 100 100`).
- Use exact geometric relationships:
  - Consistent stroke weights and gap widths across the entire mark.
  - Clean circular arcs (`A`), 45°/90° diagonals, or symmetric cubic beziers with integer/half-integer control handles.
  - Use `fill="currentColor"` (or clean `fill-rule="evenodd"` compound paths) so the mark inherits color cleanly in CSS without breaking when scaled.

---

## 3. Workflow: Plan, Critique, Construct, Stress-Test

Work in two deliberate passes:

### Pass 1: Three Distinct Geometric Directions
Before writing final production files, define **3 distinct conceptual directions** (never 3 minor variations of the same cliché):
- For each direction, state:
  1. **The Single Metaphor:** The one physical/conceptual mechanism.
  2. **Grid & Geometry:** Coordinate system (e.g., `64×64` grid, `8px` module, `4px` negative-space channels).
  3. **Why It Is Not Slop:** Explicitly verify that it avoids literal mashups and passes the 1-bit physical stencil test.

### Pass 2: Precision SVG Output & Multi-Context Verification
When building the chosen mark (or comparing the 3 candidates visually):
1. **Deliver Clean, Hand-Crafted SVG Assets:**
   - **Logomark (`mark.svg`):** Square `1:1` symbol using `currentColor`.
   - **Primary Lockup (`logo-lockup.svg`):** Symbol optically aligned to the wordmark's cap-height/x-height with deliberate kerning (`letter-spacing`) and clear-space proportions.
2. **Build a Visual Stress-Test Sheet (HTML/SVG Preview):**
   - Always render and inspect the mark across **5 mandatory stress tests** side-by-side:
     1. **1-Bit Light (`#0A0A0A` on `#FFFFFF`)**
     2. **1-Bit Dark (`#FFFFFF` on `#0A0A0A`)**
     3. **Micro Scale (`16×16px` and `24×24px` actual pixel size)**
     4. **Construction Grid Overlay** (showing the underlying geometric ratios and negative-space channels)
     5. **Physical Surface Mock** (e.g., embossed/laser-engraved on a matte 13.56 MHz NFC wristband strap or clinical badge)
3. **The Subtraction Check:** Inspect the rendered mark and ask: *"Can I remove one line, curve, or detail without losing the identity?"* If yes, remove it.

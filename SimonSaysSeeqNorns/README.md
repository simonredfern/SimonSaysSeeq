# This is a gate & CV sequencer with Ratchets & Slides

![Image 1](https://user-images.githubusercontent.com/485218/183310740-9ae29170-92d2-48f8-9fb8-c081f015f669.jpeg)

The picture above shows a [Monome Norns Shield](https://monome.org/docs/norns/shield/) which is connected to a [Monome Grid](https://monome.org/docs/grid/), a [Monome Crow](https://monome.org/docs/crow/), a [Flame U16MGTV](http://www.flame-instruments.de/pdf/Manual_Flame_MGTV_module_v100_eng.pdf) and a MIDI keyboard.

The bottom two rows of buttons are control buttons. You can use them to control the whole sequence or "put something on" one of the steps or one of the sequence rows.

-- 8th Row (bottom row) (buttons number from left to right)

1 Undo Gates (AKA Undo Grid)

2 Redo Gates (AKA Redo Grid)

3 Undo Note pitches (AKA Undo Mozart)

4 Redo Note pitches (AKA Redo Mozart)

5 ArmEuclidianRotation

6 ArmEuclidianLength

7 ArmEuclidianEvents

8 ArmRatchet

9 Randomise Gates (AKA Randomise Grid) - Pressing a sequence row on the right means a higher chance the sequence will change.

10 Randomise Notes (AKA Randomise Mozart) - Pressing a sequence row on the right means a higher chance the sequence will change.

11 PresetGates - sets some simple gate patterns (AKA Preset Grid)

12 PresetNotes - sets MIDI notes to A4 (AKA Preset Mozart)

15 Slide Off

16 Slide On

The 7th Row of buttons is sometimes used as a modifier for a button on the 8th row. e.g. to add a swing, press ArmSwing (6) and a button on the 7th row.

## Euclidean Sequencer Operations

The sequencer includes a complete Euclidean rhythm generation system using three dedicated ARM buttons:

- **5 ArmEuclidianRotation**: Sets rotation and generates the final pattern
- **6 ArmEuclidianLength**: Sets the sequence length (1-16 steps)  
- **7 ArmEuclidianEvents**: Sets the number of events/beats (1-16 events)

### How to Create Euclidean Patterns

**All ARM_EUCLIDIAN operations are row-specific** - you target the exact row you want to modify.

**Example: Creating a 4/15 Euclidean Pattern with 3-Step Rotation on Row 2**

**Step 1: Set the Length to 15 for Row 2**
1. Press and hold `ArmEuclidianLength` (position 6 on row 8)
2. While holding it, press **column 15, row 2**
3. Release both buttons
4. **Result**: Console shows "Setting euclidian_length (last_step) of row2 to: 15"

**Step 2: Set the Event Count to 4 for Row 2**
1. Press and hold `ArmEuclidianEvents` (position 7 on row 8)
2. While holding it, press **column 4, row 2**
3. Release both buttons
4. **Result**: Console shows "ARM_EUCLIDIAN_EVENTS_BUTTON: Set events to 4 and generated pattern for row 2"
   - Row 8 briefly lights up from position 1 to 4 showing the event count visually

**Step 3: Generate Pattern with Rotation 3 on Row 2**
1. Press and hold `ArmEuclidianRotation` (position 5 on row 8)
2. While holding it, press **column 4, row 2** (column 4 = rotation of 3 steps, since rotation is 0-based)
3. Release both buttons
4. **Result**: Console shows "ARM_EUCLIDIAN_ROTATION: Generated 4/15 Euclidean pattern, rotation=3, row=2"

**Final Result**: Row 2 now has a 4/15 Euclidean pattern (4 events distributed across 15 steps) rotated 3 steps to the right.

### Key Points

- **The sequencer keeps running** throughout this process - no stopping required
- **All operations target specific rows** - column/row position matters for all three buttons
- **Order recommended**: Length → Events → Rotation for best results  
- **Visual feedback**: Event count setting briefly shows on row 8
- **Rotation is 0-based**: column 1 = rotation 0, column 4 = rotation 3, etc.

See the source code for more info / up to date information.


Example usage (inspired by the TB303):


![Image 2](https://user-images.githubusercontent.com/485218/183310749-4f248b5b-d14f-4ca8-a46a-7275044307d6.JPG)

Norns (Shield) sends MIDI to create Gates on the Flame.


![IMG_4291_annotate](https://user-images.githubusercontent.com/485218/184129267-07d2316a-3495-4ac0-87ba-c7a0e8a78531.JPG)

Norns (Shield) controls Crow over USB to produce CV with glissando / slide.

On Crow, the 4 sockets nearest the USB are OUTPUTS (pitch). The two sockets furthest from the USB are INPUTS (unused).

![IMG_4290-annotate](https://user-images.githubusercontent.com/485218/184129289-01ab1354-dd41-4740-858d-358702cac016.jpeg)

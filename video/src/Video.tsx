import { AbsoluteFill, Sequence, Html5Audio, staticFile } from 'remotion';
import { Act1Problem } from './scenes/Act1Problem';
import { Act2Features } from './scenes/Act2Features';
import { Act3Close } from './scenes/Act3Close';
import { ACTS, COLORS, sec } from './lib/constants';

/**
 * Main 90-second video composition.
 *
 * Three acts composed as sequences:
 * - Act 1: Problem (0:00 - 0:15)
 * - Act 2: Features in Action (0:15 - 1:15)
 * - Act 3: Close / CTA (1:15 - 1:30)
 */
export const PrivacyFinanceDemo: React.FC = () => {
  return (
    <AbsoluteFill style={{ backgroundColor: COLORS.bgPrimary }}>
      {/* Background music — add your track to public/music.mp3 */}
      {/* <Html5Audio src={staticFile('music.mp3')} volume={0.3} /> */}

      <Sequence
        from={sec(ACTS.act1.start)}
        durationInFrames={sec(ACTS.act1.end - ACTS.act1.start)}
        name="Act 1: Problem"
      >
        <Act1Problem />
      </Sequence>

      <Sequence
        from={sec(ACTS.act2.start)}
        durationInFrames={sec(ACTS.act2.end - ACTS.act2.start)}
        name="Act 2: Features"
      >
        <Act2Features />
      </Sequence>

      <Sequence
        from={sec(ACTS.act3.start)}
        durationInFrames={sec(ACTS.act3.end - ACTS.act3.start)}
        name="Act 3: Close"
      >
        <Act3Close />
      </Sequence>
    </AbsoluteFill>
  );
};

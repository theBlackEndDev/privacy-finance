import { AbsoluteFill, Sequence } from 'remotion';
import { FeatureScene } from '../components/FeatureScene';
import { FEATURES, sec, ACTS } from '../lib/constants';

/**
 * Act 2: Features in Action (0:15 - 1:15)
 *
 * Nine feature demonstrations, each with a screen recording and text overlay.
 * Features are defined in constants.ts with their timing and overlay text.
 */
export const Act2Features: React.FC = () => {
  const act2Start = ACTS.act2.start;

  return (
    <AbsoluteFill>
      {FEATURES.map((feature) => (
        <Sequence
          key={feature.id}
          from={sec(feature.start - act2Start)}
          durationInFrames={sec(feature.end - feature.start)}
          name={`Feature: ${feature.id}`}
        >
          <FeatureScene
            featureId={feature.id}
            overlayText={feature.overlay}
          />
        </Sequence>
      ))}
    </AbsoluteFill>
  );
};

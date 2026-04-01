import { Composition } from 'remotion';
import { PrivacyFinanceDemo } from './Video';
import { VIDEO } from './lib/constants';

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="PrivacyFinanceDemo"
        component={PrivacyFinanceDemo}
        durationInFrames={VIDEO.durationInFrames}
        fps={VIDEO.fps}
        width={VIDEO.width}
        height={VIDEO.height}
      />
    </>
  );
};

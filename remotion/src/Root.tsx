import "./index.css";
import { Composition } from "remotion";
import { HelloWorld, myCompSchema } from "./HelloWorld";
import { Logo, myCompSchema2 } from "./HelloWorld/Logo";
import { Ghosty, ghostySchema } from "./Ghosty/Ghosty";

// Each <Composition> is an entry in the sidebar!

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        // You can take the "id" to render a video:
        // npx remotion render HelloWorld
        id="HelloWorld"
        component={HelloWorld}
        durationInFrames={150}
        fps={30}
        width={1920}
        height={1080}
        // You can override these props for each render:
        // https://www.remotion.dev/docs/parametrized-rendering
        schema={myCompSchema}
        defaultProps={{
          titleText: "Welcome to Remotion",
          titleColor: "#000000",
          logoColor1: "#91EAE4",
          logoColor2: "#86A8E7",
        }}
      />

      {/* Mount any React component to make it show up in the sidebar and work on it individually! */}
      <Composition
        id="OnlyLogo"
        component={Logo}
        durationInFrames={150}
        fps={30}
        width={1920}
        height={1080}
        schema={myCompSchema2}
        defaultProps={{
          logoColor1: "#91dAE2" as const,
          logoColor2: "#86A8E7" as const,
        }}
      />

      {/* Ghosty: rendered with a transparent background.
          Render to alpha MOV with:
            npx remotion render Ghosty out/ghosty.mov \
              --codec=prores --prores-profile=4444 --pixel-format=yuva444p10le
      */}
      <Composition
        id="Ghosty"
        component={Ghosty}
        durationInFrames={180}
        fps={30}
        width={1080}
        height={1080}
        schema={ghostySchema}
        defaultProps={{
          bodyColor: "#1a1a1a" as const,
          blushColor: "#f5a3ad" as const,
        }}
      />
    </>
  );
};

import React from "react";
import { MascotCharacter, mascotSchema, type MascotProps } from "./MascotCharacter";

// Variant: idle mascot (steady arms) with lip-sync mouth animation.
export const yellowMascotTalkingSchema = mascotSchema;
export type YellowMascotTalkingProps = MascotProps;

export const YellowMascotTalking: React.FC<YellowMascotTalkingProps> = (props) => (
  <MascotCharacter
    {...props}
    arm="steady"
    face="normal"
    talking={true}
    idPrefix="mascot-talking"
  />
);

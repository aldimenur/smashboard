import type { IconDefinition } from "@fortawesome/fontawesome-svg-core";
import {
  faBell,
  faBolt,
  faBomb,
  faBurst,
  faCircleExclamation,
  faCloud,
  faDrum,
  faFire,
  faGamepad,
  faGhost,
  faHandSparkles,
  faLaughSquint,
  faMusic,
  faPhoneVolume,
  faRobot,
  faShower,
  faStar,
  faVolumeHigh,
  faWandMagicSparkles,
  faWater,
} from "@fortawesome/free-solid-svg-icons";

export interface SlotIconOption {
  name: string;
  icon: IconDefinition;
  label: string;
}

export const SLOT_ICON_OPTIONS: SlotIconOption[] = [
  { name: "music", label: "Music", icon: faMusic },
  { name: "drum", label: "Drum", icon: faDrum },
  { name: "bell", label: "Bell", icon: faBell },
  { name: "bolt", label: "Bolt", icon: faBolt },
  { name: "bomb", label: "Bomb", icon: faBomb },
  { name: "burst", label: "Burst", icon: faBurst },
  { name: "cloud", label: "Cloud", icon: faCloud },
  { name: "fire", label: "Fire", icon: faFire },
  { name: "water", label: "Water", icon: faWater },
  { name: "star", label: "Star", icon: faStar },
  { name: "robot", label: "Robot", icon: faRobot },
  { name: "ghost", label: "Ghost", icon: faGhost },
  { name: "gamepad", label: "Gamepad", icon: faGamepad },
  { name: "magic", label: "Magic", icon: faWandMagicSparkles },
  { name: "sparkles", label: "Sparkles", icon: faHandSparkles },
  { name: "laugh", label: "Laugh", icon: faLaughSquint },
  { name: "alert", label: "Alert", icon: faCircleExclamation },
  { name: "volume", label: "Volume", icon: faVolumeHigh },
  { name: "phone", label: "Phone", icon: faPhoneVolume },
  { name: "shower", label: "Shower", icon: faShower },
];

const ICON_BY_NAME = new Map(SLOT_ICON_OPTIONS.map((item) => [item.name, item.icon]));

export function getSlotIcon(iconName?: string | null): IconDefinition | null {
  if (!iconName) {
    return null;
  }
  return ICON_BY_NAME.get(iconName) ?? null;
}

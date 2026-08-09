import { AlertTriangle, ArrowDownToLine, Check, Laptop } from "lucide-react";
import type { SkillStatus } from "../lib/types";

const statusMeta = {
  healthy: { label: "已同步", icon: Check },
  update: { label: "有更新", icon: ArrowDownToLine },
  conflict: { label: "有冲突", icon: AlertTriangle },
  local: { label: "仅本地", icon: Laptop },
} satisfies Record<SkillStatus, { label: string; icon: typeof Check }>;

export function StatusBadge({ status }: { status: SkillStatus }) {
  const meta = statusMeta[status];
  const Icon = meta.icon;
  return <span className={`status status--${status}`}><Icon size={13} />{meta.label}</span>;
}

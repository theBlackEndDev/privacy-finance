/**
 * Merge CSS module class names, filtering out falsy values.
 * Usage: cx(styles.base, isActive && styles.active, className)
 */
export function cx(...classes: (string | undefined | null | false)[]): string {
  return classes.filter(Boolean).join(' ');
}

import clsx, { ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

/**
 * Combines multiple class names into a single string.
 *
 * @param classes The class names to combine.
 */
export function cn(...classes: ClassValue[]): string {
    return twMerge(clsx(classes));
}

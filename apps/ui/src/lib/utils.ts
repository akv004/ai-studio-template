import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

/**
 * Utility function for conditionally merging Tailwind CSS classes
 * Handles class conflicts intelligently (e.g., p-2 vs p-4)
 */
export function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

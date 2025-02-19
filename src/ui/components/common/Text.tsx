import classNames from "classnames";

import { CSSProperties } from "react";

import useTranslation from "@hooks/useTranslation.ts";

import { TranslateArguments } from "@backend/Language.ts";

interface IProps {
    children: string;

    theme?: "primary" | "secondary" | undefined;

    className?: string | undefined;
    style?: CSSProperties | undefined;

    args?: TranslateArguments | undefined;
}

/**
 * Localized text component.
 */
function Text(props: IProps) {
    const text = useTranslation(props.children, props.args);

    return (
        <span
            className={classNames(
                // Default text theme.
                props.theme == "secondary" ? "text-secondary" : "text-primary",
                // Other specified class names.
                props.className
            )}
        >
            {text}
        </span>
    );
}

export default Text;

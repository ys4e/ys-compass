import { CSSProperties, JSX, MouseEvent } from "react";

import "@css/components/Button.scss";

type AllowedChildren = JSX.Element | string | undefined;

interface IProps {
    id?: string;
    className?: string;

    onClick?: (event: MouseEvent) => void;

    style?: CSSProperties;

    tooltip?: string;
    children?: AllowedChildren | AllowedChildren[];
}

function Button(props: IProps) {
    return (
        <button
            id={props.id}
            title={props.tooltip}
            onClick={props.onClick}
            style={props.style}
            className={`${props.className ?? ""} Button`}
        >
            {props.children}
        </button>
    );
}

export default Button;

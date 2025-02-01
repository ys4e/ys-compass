import { CSSProperties, JSX, MouseEvent } from "react";

import "@css/components/Button.scss";

interface IProps {
    id?: string;
    className?: string;

    onClick?: (event: MouseEvent) => void;

    style?: CSSProperties;

    tooltip?: string;
    children?: JSX.Element | JSX.Element[] | string | undefined;
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

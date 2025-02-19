import { CSSProperties, useEffect, useRef } from "react";
import {
    JSONEditor as Editor,
    JSONEditorPropsOptional
} from "vanilla-jsoneditor";
import "vanilla-jsoneditor/themes/jse-theme-dark.css";

type IProps = JSONEditorPropsOptional & {
    id?: string;
    className?: string;
    style?: CSSProperties;
};

/**
 * React component wrapper of 'vanilla-jsoneditor'.
 */
function JSONEditor(props: IProps) {
    const containerRef = useRef<HTMLDivElement | null>(null);
    const editorRef = useRef<Editor | null>(null);

    useEffect(() => {
        editorRef.current = new Editor({
            target: containerRef.current!,
            props: {}
        });

        return () => {
            // Destroy the editor when unmounted.
            editorRef.current?.destroy();
            editorRef.current = null;
        };
    }, []);

    useEffect(() => {
        editorRef.current?.updateProps(props);
    }, [props]);

    return (
        <div
            id={props.id}
            style={props.style}
            className={`jse-theme-dark ${props.className ?? ""}`}
            ref={containerRef}
        />
    );
}

export default JSONEditor;

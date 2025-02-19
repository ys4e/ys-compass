import {
    Timeline,
    TimelineEffect,
    TimelineRow
} from "@xzdarcy/react-timeline-editor";

import "@css/ReplayViewer.scss";

const mockData: TimelineRow[] = [
    {
        id: "0",
        actions: [
            {
                id: "action00",
                start: 0,
                end: 2,
                effectId: "effect0"
            }
        ]
    },
    {
        id: "1",
        actions: [
            {
                id: "action10",
                start: 1.5,
                end: 5,
                effectId: "effect1"
            }
        ]
    }
];

const mockEffect: Record<string, TimelineEffect> = {
    effect0: {
        id: "effect0",
        name: "test"
    },
    effect1: {
        id: "effect1",
        name: "test"
    }
};

function ReplayViewer() {
    return (
        <div id={"app"}>
            <Timeline
                style={{
                    width: "100%",
                    height: "100vh"
                }}
                editorData={mockData}
                effects={mockEffect}
            />
        </div>
    );
}

export default ReplayViewer;

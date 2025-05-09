import { useEffect, useState } from "react";

import type { Packet, ServerAddress } from "@backend/types.ts";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import Global from "@backend/Global.ts";

type ServerMessage = HandshakeMessage | PacketMessage;
type HandshakeMessage = {
    packetId: 0;
    data: number; // UNIX timestamp of when the connection was established.
};
type PacketMessage = {
    packetId: 1;
    data: Packet; // Packet information.
};

interface PacketHook {
    packets: Packet[];
    push: (packet: Packet) => void;
    clear: () => void;
}

function usePacketList(server: ServerAddress | undefined): PacketHook {
    const [packets, setPackets] = useState<Packet[]>([]);

    const push = (packet: Packet) =>
        setPackets((packets) => {
            packet.index = packets.length;
            return [...packets, packet];
        });

    useEffect(() => {
        let unlisten: UnlistenFn = () => Global.warn("unlisten not set");

        if (server != undefined) {
            // Use a websocket connection to receive live packets.
            const ws = new WebSocket(`ws://${server}`);
            ws.onopen = () => {
                setPackets([]);
                console.log("Connected to server.");

                // Send a handshake message to the server.
                ws.send(JSON.stringify({ packetId: 0 }));
            };
            ws.onclose = () => {
                console.log("Disconnected from server.");
            };
            ws.onmessage = ({ timeStamp, data }) => {
                try {
                    const message = JSON.parse(data) as ServerMessage;
                    switch (message.packetId) {
                        default:
                            console.log("Unknown packet received.", message);
                            return;
                        case 0:
                            push({
                                time: timeStamp,
                                source: "server",
                                packetId: 0,
                                packetName: "Server Handshake",
                                length: 0,
                                data: JSON.stringify({
                                    timestamp: message.data,
                                    connectedTo: server
                                })
                            });
                            return;
                        case 1:
                            message.data.time = timeStamp;
                            push(message.data);
                            return;
                    }
                } catch (error) {
                    console.error("Failed to parse JSON.", error);
                }
            };

            unlisten = () => ws.close();
        } else {
            // Use a Tauri event listener to receive packets from the backend.
            listen(Global.VISUALIZER_PACKET, ({ payload }) => {
                push(payload as Packet);
            }).then((fn) => (unlisten = fn));
        }

        return () => unlisten();
    }, []);

    return {
        packets,
        push,
        clear: () => setPackets([])
    };
}

export default usePacketList;

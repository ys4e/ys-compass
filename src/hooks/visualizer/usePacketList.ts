import { useEffect, useState } from "react";

import type { Packet, ServerAddress } from "@backend/types.ts";

type ServerMessage = HandshakeMessage | PacketMessage
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

function usePacketList(server: ServerAddress): PacketHook {
    const [packets, setPackets] = useState<Packet[]>([]);

    const push = (packet: Packet) => setPackets((packets) => {
        packet.index = packets.length;
        return [...packets, packet];
    });

    useEffect(() => {
        const ws = new WebSocket(`ws://${server}`);
        ws.onopen = () => {
            setPackets([]);
            console.log("Connected to server.");

            // Send a handshake message to the server.
            ws.send(JSON.stringify({ packetId: 0 }));
        };
        ws.onclose = () => {
            alert("Lost connection to server.");
            console.log("Disconnected from server.");
        };
        ws.onmessage = ({ timeStamp, data }) => { try {
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
                            connectedTo: server,
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
        } };
    }, []);

    return {
        packets, push,
        clear: () => setPackets([])
    };
}

export default usePacketList;

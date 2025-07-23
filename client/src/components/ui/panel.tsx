import { ReactNode } from "react";
import {
	Card,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Monitor } from "lucide-react";

export interface Host {
	id: string;
	name: string;
	ip: string;
	status: "online" | "offline" | "unknown";
	isInhibited?: boolean;
}

export function Panel({
	host,
	children,
}: {
	host: Host;
	children?: ReactNode;
}) {
	return (
		<Card className="border border-border rounded-t-none shadow-lg">
			<CardHeader className="pt-6">
				<CardTitle className="flex items-center gap-2">
					<Monitor className="h-5 w-5" />
					{host.name}
				</CardTitle>
				<CardDescription>
					IP: {host.ip} • Status: {host.status}
				</CardDescription>
			</CardHeader>
			{children}
		</Card>
	);
}

"use client";

import { useEffect, useState } from "react";
import { Button } from "@/components/ui/button";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import {
	Dialog,
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
	DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import {
	LogIn,
	Plus,
	Monitor,
	Lock,
	Unlock,
	MousePointer,
	Ban,
	Play,
	Bell,
	Server,
	Activity,
	Moon,
	Sun,
} from "lucide-react";

interface Host {
	id: string;
	name: string;
	ip: string;
	status: "online" | "offline" | "unknown";
}

interface NotificationArgs {
	title: string;
	message: string;
	urgency?: "low" | "normal" | "critical";
	timeout?: number;
}

export default function DesktopHostManager() {
	// TODO: read from config file
	const [hosts, setHosts] = useState<Host[]>([]);

	const [selectedHost, setSelectedHost] = useState<string>(hosts[0]?.id || "");
	const [isAddHostOpen, setIsAddHostOpen] = useState(false);
	const [isNotificationOpen, setIsNotificationOpen] = useState(false);
	const [newHostIp, setNewHostIp] = useState("");
	const [newHostName, setNewHostName] = useState("");
	const [notificationArgs, setNotificationArgs] = useState<NotificationArgs>({
		title: "",
		message: "",
		urgency: "normal",
		timeout: 5000,
	});

	const [theme, setTheme] = useState<string>("light");
	const [mounted, setMounted] = useState(false);

	useEffect(() => {
		setMounted(true);
		const savedTheme = localStorage.getItem("theme") || "light";
		setTheme(savedTheme);
		document.documentElement.classList.toggle("dark", savedTheme === "dark");
	}, []);

	const toggleTheme = () => {
		const newTheme = theme === "light" ? "dark" : "light";
		setTheme(newTheme);
		localStorage.setItem("theme", newTheme);
		document.documentElement.classList.toggle("dark", newTheme === "dark");
	};

	if (!mounted) {
		return null;
	}

	const currentHost = hosts.find((h) => h.id === selectedHost);

	const addHost = () => {
		if (newHostIp && newHostName) {
			const newHost: Host = {
				id: Date.now().toString(),
				name: newHostName,
				ip: newHostIp,
				status: "unknown",
			};
			setHosts([...hosts, newHost]);
			setNewHostIp("");
			setNewHostName("");
			setIsAddHostOpen(false);
		}
	};

	async function executeAction(action: string) {
		if (!currentHost) return;
		console.log(
			`Executing ${action} on ${currentHost.name} (${currentHost.ip})`,
		);
		const res = await fetch(`${process.env.SERVER_URL}/stats`);
		if (!res.ok) throw new Error("Failed to fetch stats");

		const stats = await res.json();
		console.log(stats);
	}

	const sendNotification = () => {
		if (!currentHost || !notificationArgs.title || !notificationArgs.message)
			return;
		console.log(
			`Sending notification to ${currentHost.name}:`,
			notificationArgs,
		);
		// Here you would implement the actual notification API call
		setIsNotificationOpen(false);
		setNotificationArgs({
			title: "",
			message: "",
			urgency: "normal",
			timeout: 5000,
		});
	};

	return (
		<div className="min-h-screen bg-background">
			{/* Header */}
			<header className="border-b">
				<div className="container mx-auto px-4 py-4 flex items-center justify-between">
					<div className="flex items-center gap-2">
						<Server className="h-6 w-6" />
						<h1 className="text-xl font-semibold">Desktop Host Manager</h1>
					</div>
					<div className="flex items-center gap-2">
						<Button
							variant="outline"
							size="icon"
							onClick={toggleTheme}
							className="bg-transparent"
						>
							<Sun className="h-4 w-4 rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
							<Moon className="absolute h-4 w-4 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />
							<span className="sr-only">Toggle theme</span>
						</Button>
						<Button variant="outline" className="gap-2 bg-transparent">
							<LogIn className="h-4 w-4" />
							Login
						</Button>
					</div>
				</div>
			</header>

			<div className="container mx-auto px-4 py-6">
				<div className="flex items-center justify-between mb-6">
					<div>
						<h2 className="text-2xl font-bold">Host Management</h2>
						<p className="text-muted-foreground">
							Manage your desktop hosts and their configurations
						</p>
					</div>

					{/* Add Host Dialog */}
					<Dialog open={isAddHostOpen} onOpenChange={setIsAddHostOpen}>
						<DialogTrigger asChild>
							<Button className="gap-2">
								<Plus className="h-4 w-4" />
								Add Host
							</Button>
						</DialogTrigger>
						<DialogContent>
							<DialogHeader>
								<DialogTitle>Add New Host</DialogTitle>
								<DialogDescription>
									Add a new desktop host to manage by providing its IP address
									and a friendly name.
								</DialogDescription>
							</DialogHeader>
							<div className="grid gap-4 py-4">
								<div className="grid gap-2">
									<Label htmlFor="host-name">Host Name</Label>
									<Input
										id="host-name"
										placeholder="e.g., Workstation-01"
										value={newHostName}
										onChange={(e) => setNewHostName(e.target.value)}
									/>
								</div>
								<div className="grid gap-2">
									<Label htmlFor="host-ip">IP Address</Label>
									<Input
										id="host-ip"
										placeholder="e.g., 192.168.1.100"
										value={newHostIp}
										onChange={(e) => setNewHostIp(e.target.value)}
									/>
								</div>
							</div>
							<div className="flex justify-end gap-2">
								<Button
									variant="outline"
									onClick={() => setIsAddHostOpen(false)}
								>
									Cancel
								</Button>
								<Button onClick={addHost}>Add Host</Button>
							</div>
						</DialogContent>
					</Dialog>
				</div>

				{/* Host Selection List */}
				<div className="mb-0">
					<div className="flex flex-wrap gap-1 mb-0">
						{hosts.map((host) => (
							<button
								key={host.id}
								onClick={() => setSelectedHost(host.id)}
								className={`
                  flex items-center gap-2 px-4 py-3 rounded-t-lg border transition-all duration-200
                  ${
										selectedHost === host.id
											? "bg-card text-card-foreground shadow-inner border-border translate-y-1 z-10 border-b-card"
											: "bg-muted/50 text-muted-foreground hover:bg-muted border-border hover:translate-y-0.5"
									}
                `}
							>
								<Monitor className="h-4 w-4" />
								<span className="font-medium">{host.name}</span>
								<Badge
									variant={
										host.status === "online"
											? "default"
											: host.status === "offline"
												? "destructive"
												: "secondary"
									}
									className="ml-1 text-xs"
								>
									{host.status}
								</Badge>
							</button>
						))}
					</div>
				</div>

				{/* Host Management Panel */}
				{currentHost && (
					<Card className="border border-border rounded-t-none shadow-lg">
						<CardHeader className="pt-6">
							<CardTitle className="flex items-center gap-2">
								<Monitor className="h-5 w-5" />
								{currentHost.name}
							</CardTitle>
							<CardDescription>
								IP: {currentHost.ip} • Status: {currentHost.status}
							</CardDescription>
						</CardHeader>
						<CardContent className="space-y-6">
							{/* Idle Related Options */}
							<div>
								<h3 className="text-lg font-semibold mb-3 flex items-center gap-2">
									<Activity className="h-4 w-4" />
									Idle Management
								</h3>
								<div className="grid grid-cols-2 md:grid-cols-5 gap-3">
									<Button
										variant="outline"
										className="gap-2 bg-transparent"
										onClick={() => executeAction("lock")}
									>
										<Lock className="h-4 w-4" />
										Lock
									</Button>
									<Button
										variant="outline"
										className="gap-2 bg-transparent"
										onClick={() => executeAction("unlock")}
									>
										<Unlock className="h-4 w-4" />
										Unlock
									</Button>
									<Button
										variant="outline"
										className="gap-2 bg-transparent"
										onClick={() => executeAction("simulate-activity")}
									>
										<MousePointer className="h-4 w-4" />
										Simulate Activity
									</Button>
									<Button
										variant="outline"
										className="gap-2 bg-transparent"
										onClick={() => executeAction("inhibit")}
									>
										<Ban className="h-4 w-4" />
										Inhibit
									</Button>
									<Button
										variant="outline"
										className="gap-2 bg-transparent"
										onClick={() => executeAction("uninhibit")}
									>
										<Play className="h-4 w-4" />
										Uninhibit
									</Button>
								</div>
							</div>

							<Separator />

							{/* Notification Related Options */}
							<div>
								<h3 className="text-lg font-semibold mb-3 flex items-center gap-2">
									<Bell className="h-4 w-4" />
									Notifications
								</h3>
								<Dialog
									open={isNotificationOpen}
									onOpenChange={setIsNotificationOpen}
								>
									<DialogTrigger asChild>
										<Button className="gap-2">
											<Bell className="h-4 w-4" />
											Send Notification
										</Button>
									</DialogTrigger>
									<DialogContent className="max-w-md">
										<DialogHeader>
											<DialogTitle>Send Notification</DialogTitle>
											<DialogDescription>
												Send a notification to {currentHost.name} (
												{currentHost.ip})
											</DialogDescription>
										</DialogHeader>
										<div className="grid gap-4 py-4">
											<div className="grid gap-2">
												<Label htmlFor="notification-title">Title</Label>
												<Input
													id="notification-title"
													placeholder="Notification title"
													value={notificationArgs.title}
													onChange={(e) =>
														setNotificationArgs({
															...notificationArgs,
															title: e.target.value,
														})
													}
												/>
											</div>
											<div className="grid gap-2">
												<Label htmlFor="notification-message">Message</Label>
												<Textarea
													id="notification-message"
													placeholder="Notification message"
													value={notificationArgs.message}
													onChange={(e) =>
														setNotificationArgs({
															...notificationArgs,
															message: e.target.value,
														})
													}
												/>
											</div>
											<div className="grid grid-cols-2 gap-4">
												<div className="grid gap-2">
													<Label htmlFor="urgency">Urgency</Label>
													<select
														id="urgency"
														className="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background"
														value={notificationArgs.urgency}
														onChange={(e) =>
															setNotificationArgs({
																...notificationArgs,
																urgency: e.target.value as
																	| "low"
																	| "normal"
																	| "critical",
															})
														}
													>
														<option value="low">Low</option>
														<option value="normal">Normal</option>
														<option value="critical">Critical</option>
													</select>
												</div>
												<div className="grid gap-2">
													<Label htmlFor="timeout">Timeout (ms)</Label>
													<Input
														id="timeout"
														type="number"
														placeholder="5000"
														value={notificationArgs.timeout}
														onChange={(e) =>
															setNotificationArgs({
																...notificationArgs,
																timeout:
																	Number.parseInt(e.target.value) || 5000,
															})
														}
													/>
												</div>
											</div>
										</div>
										<div className="flex justify-end gap-2">
											<Button
												variant="outline"
												onClick={() => setIsNotificationOpen(false)}
											>
												Cancel
											</Button>
											<Button onClick={sendNotification}>
												Send Notification
											</Button>
										</div>
									</DialogContent>
								</Dialog>
							</div>
						</CardContent>
					</Card>
				)}
			</div>
		</div>
	);
}

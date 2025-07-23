import { LogIn, Server, Moon, Sun } from "lucide-react";
import { Button } from "@/components/ui/button";

function toggleTheme({ theme, setTheme }: HeaderProps) {
	const newTheme = theme === "light" ? "dark" : "light";
	setTheme(newTheme);
	localStorage.setItem("theme", newTheme);
	document.documentElement.classList.toggle("dark", newTheme === "dark");
}

interface HeaderProps {
	theme: string;
	setTheme: (theme: "light" | "dark") => void;
}

function Header(props: HeaderProps) {
	return (
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
						onClick={() => toggleTheme(props)}
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
	);
}

export default Header;

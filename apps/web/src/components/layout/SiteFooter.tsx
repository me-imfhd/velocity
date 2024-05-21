import Link from "next/link";
import { Shell, buttonVariants } from "@repo/ui/components";
import { Company, mySocials } from "@/src/lib/config/site";
import { Icons, LucideProps } from "@repo/ui/icons";
import { cn } from "@repo/ui/cn";

export const SiteFooter = () => {
  return (
    <footer className="w-full border-t bg-background">
      <Shell as={"div"}>
        <section
          id="footer-bottom"
          aria-labelledby="footer-bottom-heading"
          className="flex flex-col xl:flex-row lg:flex-row md:flex-row sm:flex-row items-center space-x-4"
        >
          <div className="text-left text-sm leading-loose text-muted-foreground">
            Built by{" "}
            <a
              aria-label="Kickflip tutorial on YouTube"
              href="/twitter"
              target="_blank"
              rel="noreferrer"
              className="font-semibold transition-colors hover:text-foreground"
            >
              Fahad
            </a>
            .
          </div>
          <ul className="flex-1 flex flex-row space-x-3">
            {Company.map((link,index) => (
              <li key={index}>
                <Link
                  href={link.href}
                  className="text-sm text-muted-foreground transition-colors hover:text-foreground hover:underline"
                >
                  {link.title} <span className="sr-only">{link.title}</span>
                </Link>
              </li>
            ))}
          </ul>
          <div className="flex items-center space-x-1">
            {mySocials.map((social, index) => {
              const Icon = Icons[social.icon] as ({
                ...props
              }: LucideProps) => JSX.Element;
              return (
                <Link key={index} href={social.href} target="_blank" rel="noreferrer">
                  <div
                    className={cn(
                      buttonVariants({ variant: "ghost", size: "xs" })
                    )}
                  >
                    <Icon className="h-4 w-4" />
                    <span className="sr-only">{social.title}</span>
                  </div>
                </Link>
              );
            })}
          </div>
        </section>
      </Shell>
    </footer>
  );
};

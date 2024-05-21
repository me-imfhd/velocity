import EmailProvider from "next-auth/providers/email";
import DiscordProvider from "next-auth/providers/discord";
import GoogleProvider from "next-auth/providers/google";
import GitHubProvider from "next-auth/providers/github";

import { PrismaAdapter } from "@next-auth/prisma-adapter";
// import type { Adapter } from "@auth/core/adapters";
import {
  NextAuthOptions,
  type DefaultSession,
  getServerSession,
} from "next-auth";
import { db } from "@repo/db";
import type { SessionModel, UserModel, z } from "@repo/db";
import NextAuth from "./next-auth";
import { CustomsendVerificationRequest } from "./sendVerificationRequest";
import { redirect } from "next/navigation";

export type { Session, DefaultSession as DefaultAuthSession } from "next-auth";

export const providers = ["discord", "google", "github"] as const;
export type OAuthProviders = (typeof providers)[number];
/**
 * Module augmentation for `next-auth` types. Allows us to add custom properties to the `session`
 * object and keep type safety.
 *
 * @see https://next-auth.js.org/getting-started/typescript#module-augmentation
 */
declare module "next-auth" {
  interface User extends z.infer<typeof UserModel> {}
  interface Session extends z.infer<typeof SessionModel>, DefaultSession {
    user: DefaultSession["user"] & {
      id: string;
    };
  }
  interface Profile {
    id: string;
  }
}

// if (!process.env.GITHUB_ID) {
//   throw new Error('No GITHUB_ID has been provided.');
// }

const useSecureCookies = process.env.VERCEL_ENV === "production";
const cookiePrefix = useSecureCookies ? "__Secure-" : "";
const cookieDomain = useSecureCookies
  ? process.env.HOST
  : "localhost";

export const authOptions: NextAuthOptions = {
  pages: {
    signIn: "/sign-in",
  },
  debug: process.env.NODE_ENV !== "production",
  secret: process.env.NEXTAUTH_SECRET,
  cookies: {
    sessionToken: {
      name: `${cookiePrefix}next-auth.session-token`,
      options: {
        httpOnly: true,
        sameSite: "lax",
        path: "/",
        domain: cookieDomain,
        secure: useSecureCookies,
      },
    },
  },
  adapter: PrismaAdapter(db),
  providers: [
    EmailProvider({
      server: {
        host: process.env.EMAIL_SERVER_HOST,
        //@ts-ignore
        port: process.env.EMAIL_SERVER_PORT,
        auth: {
          user: process.env.EMAIL_SERVER_USER,
          pass: process.env.EMAIL_SERVER_PASSWORD,
        },
      },
      from: process.env.EMAIL_FROM,
      sendVerificationRequest(params) {
        CustomsendVerificationRequest(params);
      },
    }),
    DiscordProvider({
      clientId: process.env.DISCORD_CLIENT_ID!,
      clientSecret: process.env.DISCORD_CLIENT_SECRET!,
    }),
    GoogleProvider({
      clientId: process.env.GOOGLE_CLIENT_ID!,
      clientSecret: process.env.GOOGLE_CLIENT_SECRET!,
    }),
    GitHubProvider({
      clientId: process.env.GITHUB_CLIENT_ID!,
      clientSecret: process.env.GITHUB_CLIENT_SECRET!,
    }),
  ],
  callbacks: {
    // async signIn({ user, account, email }) {
    //   const userExists = await db.user.findFirst({
    //     where: {
    //       email: user.email, //the user object has an email property, which contains the email the user entered.
    //     },
    //   });
    //   if (userExists) {
    //     return true; //if the email exists in the User collection, email them a magic login link
    //   } else {

    //   }
    // },
    async session({ session, user }) {
      if (user) {
        session.user = session.user || {};
        session.user.id = user.id;
        session.user.name = user.name;
        session.user.email = user.email;
        session.user.image = user.image;
      }
      return session;
    },
  },
};
const handler = NextAuth(authOptions);
export { handler as GET, handler as POST };
export { getServerSession } from "next-auth";

export const auth = async () => {
  const session = await getServerSession(authOptions);
  const user = session?.user;
  return user;
};

export const checkAuth = async () => {
  const session = await auth();
  if (!session?.id) {
    return redirect("/sign-in");
  }
  return session;
};

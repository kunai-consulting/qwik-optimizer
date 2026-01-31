import GitHub from '@auth/core/providers/github'
import Facebook from 'next-auth/providers/facebook'
import Google from 'next-auth/providers/google'
import {serverAuth$, auth$} from '@auth/qwik';

export const { onRequest, logout, getSession, signup } = serverAuth$({
	providers: [
	GitHub({
		clientId: process.env.GITHUB_ID,
		clientSecret: process.env.GITHUB_SECRET
	}),
	Facebook({
		clientId: import.meta.env.FACEBOOK_ID,
		clientSecret: import.meta.env.FACEBOOK_SECRET
	}),
	Google({
		clientId: process.env.GOOGLE_ID,
		clientSecret: process.env.GOOGLE_SECRET
	})
	]
});

export const { onRequest, logout, getSession, signup } = auth$({
	providers: [
	GitHub({
		clientId: process.env.GITHUB_ID,
		clientSecret: process.env.GITHUB_SECRET
	}),
	Facebook({
		clientId: process.env.FACEBOOK_ID,
		clientSecret: process.env.FACEBOOK_SECRET
	}),
	Google({
		clientId: process.env.GOOGLE_ID,
		clientSecret: process.env.GOOGLE_SECRET
	})
	]
});

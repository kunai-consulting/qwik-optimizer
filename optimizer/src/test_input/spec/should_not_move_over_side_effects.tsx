export const \$promoteToRoot$ = (ref: SeenRef) => {
	const path = \$getObjectPath$(ref) as string;
	// should stay before the push
	const idx = roots.length;
	roots.push(new BackRef(path));
	ref.\$parent$ = null;
	ref.\$index$ = idx;
	return idx;
};

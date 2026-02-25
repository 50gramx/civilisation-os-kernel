# The Atomic Action: Fueling the Accountability Engine

For the Dynamic Symmetry physics engine to work, it requires a fuel source (Activity) that cannot be cheaply counterfeited. 

The user asks: *What is the minimum measurable action in V1 that cannot be automated cheaply, produces real signal, and is domain-agnostic?*

---

## 1. The False Atoms (What Fails)

If you pick the wrong atomic action, the system is farmed to death in 24 hours.
*   **Failed Atom: The "Like/Upvote/Reaction."** Too cheap. A Python script can generate 10,000 likes a minute. 
*   **Failed Atom: The "Page View/Time on Page."** Too cheap. A headless browser can simulate reading a page for 5 minutes.
*   **Failed Atom: Financial Staking.** Breaks the "Pure Native V1" rule (requires liquidity/fiat).
*   **Failed Atom: Captcha Solving.** Proves you are human, but produces zero social signal or value to the domain.

---

## 2. The True Atom: The Cryptographic Vouch-Bond (The "Endorsement")

The core atomic action must be **relational**, not transactional. It must inherently tie the actor's fate to the object they are acting upon.

**The Atomic Action is: The Vouch-Bond.**

A Vouch-Bond is not a "Like." It is a cryptographic signature stating: *"I stake a percentage of my accumulated time/accountability score on the validity and value of this specific Entity."*

The "Entity" can be domain-agnostic:
*   In `creators.local`: It is a piece of content (a video, a post).
*   In `food.local`: It is a delivery receipt.
*   In `civic.local`: It is a governance proposal.

### Why It Works (The Physics of the Atom)

1.  **It is Costly (Stake Cost):** You are not spending money; you are locking your accumulated *Accountability Score*. If you have a score of 100, and you Vouch-Bond a new creator's post, you lock 5 points of that score. While locked, you cannot use those 5 points to endorse anything else, nor can you use them to onboard a new user. 
2.  **It Produces Real Signal:** Because your score is locked (and at risk of slashing), you do not Vouch-Bond randomly. You only Vouch-Bond entities you actually believe in. The resulting graph of Vouch-Bonds is the highest-fidelity signal of quality on the internet.
3.  **It Cannot be Cheapest-Automated:** A bot farm can read a page, but a bot farm has an Accountability Score of 0. They have no cryptographic weight to lock. Generating weight takes *Time*.

---

## 3. Defense Mechanics (Surviving the Real World)

If the Vouch-Bond is the atom, how do we stop smart attackers?

### A. The "Farm Rings" Defense (Graph Distance)
*   **The Attack:** 10 attackers sit in a room, onboard each other, wait 3 months, and then exclusively Vouch-Bond each other's content to inflate their scores.
*   **The Defense:** *Endorsement Diversity Squelching*. The Accountability formula measures the shortest path between you and the person whose entity you are bonding. If you only bond people 1 hop away from you (your immediate clique), the system yields a multiplier of `0.1x`. To get a `1.0x` yield on your activity score, you must bond entities from people 3 or 4 hops away (strangers in the domain). To grow, you *must* interact outside your cartel.

### B. The "Inertia" Defense (Decay)
*   As discussed, the formula is: `Score(t) = base_score × e^(−λ × inactivity)`
*   If an early adopter Vouch-Bonds 100 great creators and then goes to sleep for a year, their score bleeds heavily. They lose their right to onboard new users. They lose their voting weight. The active citizens always displace the sleeping nobility.

### C. The Slashing Triggers (Objective)
How does a Vouch-Bond lead to a slash?
*   If an entity you Vouch-Bonded is subsequently purged by the network via objective thresholds (e.g., it is flagged by 90% of the domain's high-weight users as spam, or its structural payload fails a cryptographic signature check), your locked stake is burned. 
*   **No Subjective Slashing:** You are never slashed because someone disagreed with your opinion. You are slashed because you Vouch-Bonded an entity that mathematically proved to be a protocol violation.

---

## 4. The User Experience (Invisible Physics)

The user does not see "Vouch-Bonds" or "Decay Formulas." 

**The UI is simply:**
*   A button beneath a creator's post that says: **"Back This."**
*   When hovered, a tooltip says: *Backing this uses 5 of your Citizen Points. Points return slowly over time.*

That's it. 

The Civilisation OS runs entirely on the physics of people choosing what to "Back," knowing their reputation is tied to that choice. This is the atom.

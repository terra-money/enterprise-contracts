import { Coin } from "@terra-money/terra.js";
import task, {Deployer, Executor, Refs} from "@terra-money/terrariums";
import {Signer} from "@terra-money/terrariums/lib/src/signers";

const WARP_CONTROLLER_ADDRESS = "terra1mg93d4g69tsf3x6sa9nkmkzc9wl38gdrygu0sewwcwj6l2a4089sdd7fgj";
const ENTERPRISE_FACADE = "enterprise-facade";

task(async ({network, executor, refs }) => {
    // await createWarpAccount(executor, WARP_CONTROLLER_ADDRESS, 100_000_000);
    //
    // await createMigrationStepsOldWarpJob(refs, network, executor, WARP_CONTROLLER_ADDRESS, "terra1a9qnerqlhnkqummr9vyky6qmenvhqldy2gnvkdd97etsyt7amp6ss3r237", 20);
    //
    // await executeWarpJob(executor, 1);

    await createMigrationStepsOldWarpJobMultiple(
        refs,
        network,
        executor,
        WARP_CONTROLLER_ADDRESS,
        20,
        [
            // "terra14760vrvj0qn9ry40tkauffjp5t78xelmflhlex66wz0n5x8uz3vqe0kxef",
            // "terra1nulflfswh70rsfz7qj3eypdrxgys3jj36zvzcjevdqrl6w3s5w3q4jugws",
            // "terra15l2v0mdf73ryrr2hjwtrjguvpjl5nn0tycty8pcj0j3nd60d8p8sl9g5lw",
            // "terra1290d4q6av48d3r8y99s4d5fqr5k75hn7l7ytz27pu3fqvg3f4jhsqr9vju",
            // "terra1rh80t734ftq2w0mhh0w0drqsy2076qu845zcy72mjurzktdlnd5qn9ndtr",
            // "terra190dpn5p38c7ncxp279pw0jqnx9qh2qfzaxwmfwp0fhqtlezew2nqre9f2t",
            // "terra17c6ts8grcfrgquhj3haclg44le8s7qkx6l2yx33acguxhpf000xqhnl3je",
            // "terra16p0nmsy5wuvpdmvxz03gsfsxpdsyzyj2088dcqery49gdsscelqqgqes2y",
            // "terra1ns4ndc86qjs5wsggr4rcjsxgeveckg6gu4l4jggw99tsy3xrkrpqx2n46m",
            // "terra1l8nl9tdz3qcw4mygg97vl5th6du0hl29ay8l3jfu47x7znyyfphse9dlql",
            // "terra1ja0v4twd3rxqg83nugrnllszdtge28rrxkggazzcjn6axd9g8qhsdqsw40",
            // "terra1d79l8q0ykxracurejyk7ym0hnmhf9ndpd09vqjfgchwpgtjwjp8qluu2k4",
            // "terra1a43xtm8h6fk3xzdj57hv0lru4w5399lz9y40m7608lwz52aqyyjs3tzsqd",
            // "terra1xp8xmxycaps0hs9dhd2nwtmpv083kyepq6f7qhw4x5c602wklm2qc7m9gf",
            // "terra1dfyg2m6pc7c8fxp6q2qf4supeqlqla95qs4dh3dkv44alhmhpjuqdcwszn",
            // "terra1c3hnheg7geayfgsv5c9nmaqlgwrmkkvazwp60xese38q6h3ug8eqvv3kd5",
            // "terra1c77cqx3vdfxzg90fu2nrc23u38kvv0lj5en48y3lauxwmxprn2xqntj50y",
            // "terra1znrx27vrvx7ldj3tj3plgrvk0a2v0ku9q6f7vwq60ajxzlyx7lcqdlth4e",
            // "terra189zwwfvlqesz54aqu6xdt50ehyn57d536k384gm9dgwvc0lcffhqytajkf",
            // "terra1045464y7vmsc97ydlkzwhv6qz6wa7zkfmhr8e5xu56rmhyfawjuse2hhn4",
            // "terra1y82zczv6julsyjvg6lwzwyr6dyxh4egxc5rqq6ry3aylwz39dwlq3k8pf9",
            // "terra1jzca4gjac60ls74slsf9r9fjluvk45jk0muxmflz6vxzqdq5amnqaz2y92",
            // "terra1r7wzh4pampmf69wls8f2e05uc246g9uxgyr4naf27e3tvrgqcgas0ahkmu",
            // "terra1k8ss9yhmkpgvccpjdvnrj00qpvkr9f6vt58wedtexhk95xnunc4sdvjm2h",
            // "terra1yu0jagjqpaeuswdlqnnsexltcs4t3zl4luey3z7xygq4z2mw5f5sxlyecj",
            // "terra1hasy9hfn0kpqkhnl9xrg26g24yle7249dpcrj4wj3vhmkxmu2hvqj57tn9",
            // "terra1syvtuzpmf77yvl554g4u95vj9rnhdlq345l6arl6sc26ngcp0a0sg6fx77",
            // "terra18sa2lsr5jvwfkjshpgh24238re8pg4lqrs5hya65fcvqz0walngscyawce",
            // "terra1chzhfnuh3mxl8ry5n3ph820cxt2q2ca98xdskwsfqj6squ9wvpvssu0zfj",
            // "terra1ydkvywwnl3j84tcntcwjmzgjc5u2vrqpcyjzn3slvwcpjke6nzhstm5a0g",
            // "terra1qmedptfkctta4vayapxke3aem00vtpmaqa75mg622yrhqz8d2knqr3e74c",
            // "terra1ysfmzka4yacrjpdryxjslgwf2amhgvayp8use2x66h92px0lvgrqn4vhjq",
            // "terra1uzuvufpvwpjt62prl9lxtkpccl79slys8p2jhc9afpn8rtlsczhqsehs0q",
            // "terra1sm76wl58y3par4tmzjwlv0fuzrqgrjnsvpxqj27x2sj38ee2trpswgjdjx",
            // "terra1q2j9ezy43gu8sd5juh7ayhhwlevq437fyzl24fhw2wp2asvfhcgq6pa4vn",
            // "terra1ghhjsjtnmwyhzmtnzh8yx2yrkzwn8wc36vpkxkhgr6fk2tt7zzsqjkqy5r",
            // "terra1xtx73wxh2echhl99shdr0mpfhal4mesenah7qfzdktxjqzuxkr0qfzeqxh",
            // "terra1gl0cxwd34zaed0vd04fjqt04x55xl0pc0sctnpyjsz6uyn7m4mls6rwjx7",
            // "terra1hu6hffr8wdl2mxp2qv7qv5vj84je34msa2acuzuxd4um8up9a8zskchgmg",
            // "terra17z87eqstn3d3tt8pypl74wrdrwktt7rl9va7z88pln7ke5c0srnsmtaven",
            // "terra142nfhgw6v9mp5myrhv82gacg455qz0mgkzjernj7jmz3j47t9wlqxn5auf",
            // "terra19jmyywzzatecsvy72ulmsmt5kjser3ep5chv0a4hlyyz28k3khnqvlprm0",
            // "terra16rfgrqsvwfvhrxmsyxzp3l827hm3nzwzzc27gdjmk2aunjmx38xskf7l3s",
            // "terra1mey3n7rzf69rvtetrnrwe09de0jc55p6x09juhql47vrrh5hkkyqc6hy7l",
            // "terra1v72eja3zpl037zdw2aa42wg96anwcufz8xctfqwwl0gxfy85mqpqffnra2",
            // "terra1d7drvn8w78a2na8492at3ndsaxr643edq6sr07n4m87x2zx7k98q2uhj5n",
            // "terra1sa47q8myqdpw3pcdwahef99nmp3rawvhs87tqmj92akrtjjl6vpqzd2s0k",
            // "terra1um8c94suatpwmwavcgp8j052tzhngtn7xys557yaraukgc9was7szzkv95",
            // "terra1ylmjvlayldxler4s9rhm6ycny2l62x3rgyfjrjs6n6p4sr98rj7q22w0ek",
            // "terra1ksehuwyd3t258m53nvru6jznu76gu29pgv5euymjgsh6u0tdgjvst0qldk",
            // "terra19qtmchmgfrhr0np2rlex3netut0agxllj6qehmh8aqa5pwq4578s64csyk",
            // "terra1nmm7qg44ml73emgszdsded9qn2vthmstn6z900wjwz0a38jlyujqkh6p7r",
            // "terra1akskaufwjcs34nwrsgxxgqswflppekdk7mla3za6hh3nlnsw0sasuh0ftz",
            // "terra158ffgz08fq2ghuu3lv8sm2z8py3sp6v7pahl85j766wznqyfmgmqvx5jdq",
            // "terra1vymxz0qxpczxgupu98ghsfe5phzug6rdcp62rz3a503c57dmctnsr2vwad",
            // "terra1kse252t682zp97j8m6548266n7cmm7ysqt7gsy6cg93vp65hqx8qh82jcc",
            // "terra1thstdp677jt2n3mn98rn038a2qgvas7eys20sm44q3ez2rthgzjshvgmg4",
            // "terra195xryp6xltt64cmkmtcp8qtudjp8wqszzcyhv8zm9457r0u3prrsz355g8",
            // "terra1j58aeyxxcpe0l7ft2tvyfrzjc70hp4p6p46mmg3ug4x3qjma8sfsfxcfut",
            // "terra1zx3ap27rq0vqavla6afmm25gc9vkaem4qurce4p0ek0vqdyan63sthudf6",
            // "terra1r0td23qctgvww4glsn9r3x2llnsm2vv0wgha6wetl0t0hwgz8mrqajnn0x",
            // "terra1n6mm9xmk6nk4j4q2ms8zvlt9mp340t6qtwnwc0r7pcfdq30nhdms4w07jt",
            // "terra1fpjzgvrgmagnm02wdcwz7drpcrcvrc336nn9upeqz0lgwm006f0q4d6qxk",
            // "terra12vned759qle4kgq9xne9krwkgavgqmt4watrx8x4euks7wpcf9ys9x3daf",
            // "terra18x6fq9m9r9c4c0ev6g6epv9jeqg5qgyls37j40nl40lr78zz5j6s8zgr68",
            // "terra1jw39mglv63mtmkgd4synsyl50cwvxsp9wc9mjva7u2cej7qxu7ks2e2hd8",
            // "terra1ragx7ypjt2haf9956z55phr3f2degljf7n5gq5las05u5uly94wsyejml0",
            // "terra1yl6dkpe9tcnlnn6gxnqry8ny42fcmwwh4wv6fqe7vjl4qgveaqyqsdlnkp",
            // "terra18p53aa3s0z22dnl74kepysafy6mm5ahaefwzgaanj32kvpuyzdkqs5rw89",
            // "terra1w3xuztwy0xgyhfwz9elsflt2s64rflffthee8mxvlln2vzdqdvgqa38s73",
            "terra1am2ycwuq970ajw5vqr8gnp63y63ekpkgxpndxm2r8vkp4avx6paq37efm6",
            "terra1hl7efa292ayezmljhj2rsw84hf6e32rzycfwjcr7c58z5fwysufq8yrcyr",
            "terra1ch6n356dgx39rtsmu0cva2c2vj0ql4nwquys0s0fystmx5utx5zs7xh7w5",
            "terra1r73lz8crnx2ms0nrqtdpxel908vz3pwh9yjnt8ug8knqlxgx60hq5zrtph",
            "terra17c7txpdwh6rhs4n2mu75vp26g87yzsmyxydqv9zdnjnjtnrjcn8s8jfl94",
            "terra1vmaggjxf4u3ft5upz3e9wuwu9msl34szuept4mpjwtnud4l65eaqvxyh5u",
            "terra18kp32m7jfvcx43m8ymr87ut3cavcjx6s6y830cgmz2hlx505xjwshhw7v8",
            "terra18h3lrcmcavaggmj6ylyqd9030xae746lk582z90u3xtk5l3303mspg6ffk",
            "terra1x8wyy2tmvwn5nm23maxry80mkpxn65x2ghs0q3ktnk5y62wj5x7s5vsg79",
            "terra1lwypcpsmgferepq5pm3zj7xswy44t9jjnf9u9vaprzfywkmyyhzshw74nk",
            "terra19jec7mdzt49707y30d0cfsq2t0hg53up03lv4rgqk5ud9e6xjq9qztcaeh",
            "terra12w4lmmk3a9edhp6p2xm4t0l8t358u4x6l3aq7jng73dvsztfyngq53p62f",
            "terra1rwyw7azmqpm80ahm022jvy6nrztlj6juseyw8lfkv0ppu9r33czs52tsh2",
            // "terra1dsed62qn5mlrrskmqa7t79e6nnfhffl5uqn2rvtpnjp77ekngxesq0hlw3",
            // "terra12axuj7zjmtg0v4rfprn93e7drhm3f2k9v5vy79s8wl6c34hdptpqf372py",
            // "terra1j6jdykxkrwcqd7mvqwf3zrx44caxjvpwz4fpjyz9ge6j2lqvnh7q5xahu2",
            // "terra1exj6fxvrg6xuukgx4l90ujg3vh6420540mdr6scrj62u2shk33sqnp0stl",
            // "terra1ckh55ww6vmp46au6rqe4s444up3rmsaflcz3e03jzxypzfqr5kgq3hw33e",
            // "terra1s0g7hychwkurysgtlmsjkjrx7hy62mdms048thxxj83wtlwwmskqktuljx",
            // "terra18f9eklmlvk9hl78e2kwneymunppu8ncvhpsrn2zhc085sah2w3rsa5gssp",
            // "terra1zsxq90mrkxjnr9zgev4ulyg3vs0fk4afcrw9zy6agzrdnwv7yevser7pye",
            // "terra1cyjdz0fnwxc0hdyjfdrtl07jvuxyx4pat7v3rc9z5rtgpkn70vfqzeg8uc",
            // "terra1wwarpmedyce70x6l8nkn76rwld76wt3ej9cv6aghk0504p5xmwwssxdp92",
            // "terra1sfx94yzppppke2z8zj0ce0fr7t56u0vznh6mln4p9078z28uf6aq6fgwhy",
            // "terra1f43s2vecnmlany8q87e6qafj6mnu249k0yqzg477qsdhzsv39dhq5kxdsj",
            // "terra1kpk0jdw2hmgzyt7fju499us87fdr9jekd6cm3qdc2lha5snmdaqqgjlds4",
            // "terra1uqjkhlw3ggnfhs690m56zunrctm60wafl93fyy6wjn4tlmag2yvsn240l4",
            // "terra1mjhu6tnf8djhnnnntfzs3s58trh8qgp57g3ppx90xxrhh3u36x6qzej956",
            // "terra1r5t3e3n82adra7aq633frd68e5qezwvquts5tgqe5x2cx7tl52sszvqztl",
            // "terra1nm3q7q595msc9lqv4ndalr9q0vqvvck9f0e0nn5skle273y7c38sg7834k",
            // "terra14403rxh3cajmk0ul9fly8dq45ftdux9u3pjsx0lue6qtkadaqaksxu25tr",
            // "terra18ufp4m89zzclg26awmak8k5yq9tu75lyf96nhwr2a779rfpdwfaq84vprh",
            // "terra1wzyquj5the5en32f8j4eev9r6yyvt9dm945qqul2p5wtayxkz94sqrl0y3",
            // "terra1v52jjvqm24sgerajzatf6emx53mxhsjrxzkvtaj9zusjapex2dlq07llyu",
            // "terra164axm975pxtlnd0l09k3l02er0pedgg2qmq7sd3d9p8enm8zdexqe9tnn5",
            // "terra1uzafld6nray0jck24p6xp9lyr46pnskn0ss7vpdj3l0t0arvmxls0pahtw",
            // "terra1mapju8pck5ppzu0sfexcyvwx0j8k4jfvzgdphhztp8k398slfftqjlf595",
            // "terra1h2xaf7l2yk5uc37hu6k8ltgvxhflt2m0jtr27rswvfj0fctynk6qsghacj",
            // "terra1jcrz0wu8jxzd8vs6448j3ue7p9qs4cpjzxgccjxxcfumuer7l6zqfmtwkv",
            // "terra1c7u0c7e9c8u5rjaupnvf7pndd5e2d4hrap68jpney3q74eht0vkq7apsct",
            // "terra1g5q5vsakkx6ue63res57368s3kqfwg8v82kyvhq9px7c0yramdrqzfxmg4",
            // "terra1hmjmr0sc5r8rrdpfwghkxq8yd3h93ya4ufxcny0s5qqlygmwdelqcwzjzj",
            // "terra1m8yppctc5x6u4hp4xnw7e9yyg9gc96agdx2t9umfrxq9t0p6qywq3j4yzs",
            // "terra1gcgg3nte7ufmy9elvzdky47fkl264w6qqg5hke8mnsmgf3saktlszn2ljq",
            // "terra1v4sjvr7v59p33h3tvmu98acswvy4zhkq4fs0wyaf9v2kugj0nxuqssfpnn",
            // "terra1szv4tucduxym52q305v3x6dejyqsxhqt63vjy2rpjff8ap40jdkskkx3hy",
            // "terra1w2q5am5agfwpl7v5hwczc562v807ehd0aun0va3tzfteswc9drwqu3h9cs",
            // "terra1fq6stx9yqrvgsawevljpy83sf4680pz7guq929h852c384zjt36qlkznya",
            // "terra18mt89jf9wmveglhc2790q8ur4ck2hmaj6tvaaceggn5w4yhs6kps5809u0",
            // "terra19jxrmh029kxg3e4fc4k8seyhankfy9gksnyg80y9kh86a9yv6g0q6yfjgf",
            // "terra16vl35edwt5c2904l7zlezv5kr6fwzjk78mc6wmf9rzutxxc7nfksymzuce",
            // "terra12znpyklu8zrkkuy66fu0ruhaj5czeqw2z67mzwqll2k9grxcaf5ss6gecc",
            // "terra1jkk23vzrqjdcr2ku29d6khyey9k4m2s8kdclms7ezcdy4zpunvnsg88s9d",
            // "terra1srxyjj0795nuksupdcuy35v547rgqdcvdjcrx857yf2z7kf4mtwqk5jurc",
            // "terra12waku295pu7w5ly7v3henvxdsgn8mckxxymu6cczfga38tzh8pgse2w722",
            // "terra1c6mvalz5npsyvlqly69wnazg796y0pu4ge4za9euj9ehnh790j7s85q5yp",
            // "terra175v68wt6jtrp8p0xw0rm4cm0ygvcea40mtd3vywd6l28eral7stsgf84zn",
            // "terra1yq3z7xmtxzqwf0zarkj7yjmpaldrjgufafava6farczztlz8lrnqcy42rt",
            // "terra159q4e7zl84hzkwy95kl29accklrxpth4zcuz8m87p4nvykpszrtq5qfgfe",
            // "terra1f2l9t3wr6uljjxexjlrqrrx22ehh0r7vsemdsyy49kqrmmkmxucsta2vwr",
            // "terra1yv5fyftazjsy3uslzwrsaqcahn8mht87kf7jzlh50yfnu7mqxymsja06dz",
            // "terra14tfwjfr35clkugfusqa9868l74tymd669u8ellw492n50rsuvv8q94lh9a",
            // "terra14zs05y3hc3ran6wyvuj8fr2kg3v6vpknzen6n9xdt9exf68w6sjq36975d",
            // "terra1ae2fzd44p6ler8csyfclhhav34kzz45pjd8ymy4ye3zjgc984cvqw5ystm",
            // "terra1fm0hjnra9mm4tfwmdwpynvvw7v5z6c9szdec5g0dfysvmrj7mlqs24f9zu",
            // "terra1mw4x2h54ft4kyv3v3thsln8vnzxah9hutgludn2hfzlzu33x5rzqqzx58z",
            // "terra1gxw5u64xud9y5dv8y3uk4x3cftf3a055v6tn5puksxq7aezcag0q5nwx30",
            // "terra1z2sgjtez2tuqrtdvz7g7yhxj8mc7x6v877fvt4940zqp3gszen6s9gxu2r",
            // "terra1vm3v334jttp7ur7n6cqcq3w6xq78t49q5n4sw4a7jkddzhfuqntqpcgamt",
            // "terra15ysxwg90y3yy3hrd3vyf6smf7lk9a7an8q0fryc48ssr3j7werdqr8n9zw",
            // "terra1c88xa92l0rewxs27dv5r7j98kuzyz2er9vk699nqzlksy838tt6qq3xupw",
            // "terra1qlcwa4k7zpx7ep2uh4cstv7gumjwk2lg0dtavx7h0x8sr0hprumqurjevx",
            // "terra12agp7scuht4qdtpldyen0l4cxz5xe2q0hws9hk4acw5hkdr2dx6qc8cwu5",
            // "terra1z0l5knj0laqc45usknemxa8jmpveqe9q67wvxga4u7z8tulhy65q95p43d",
            // "terra1nglxvkhd98ms3q0uyqwkk9lqpv084e4y4kfy5dnmu9zls92vh2tqa68x2j",
            // "terra1x74qup5ru9e7wp9wltc023rkqk9xrlcqcefqyz969gy2hylwjqqqakf3j8",
            // "terra14m8emqepq5lkvph2mntu5wymkkss2zlhrsmwc4hjgl0pewrdjd6q2pqcuw",
            // "terra1xk2u08ddekdry87a2qufh2w3gu4gvfe6akprkg09vdndek33jzxsvr0unp",
            // "terra16psuek9ag5zq0h6uu6xuw92jay08rh8286f4tq9am0ec0dfp57wszrxrau",
            // "terra1ec56pjxtcw279xr995ex5jyzde9wflt4jdanl6czk2773yesqjlqgwjjpt",
            // "terra14z729cpgeelm9u8xv58t3z2eda3cg0g0h3szpywdgkyyaln9djjs7t48e4",
            // "terra10z4a0utxu4tq2zsgl9fdw4h8cte7vyr2z6qdh3gnrqsgfpyy6anqlj95mp",
            // "terra17p3dzgw37xw3xa3mee4ph459zz9pnw7qjvz49rxz7lx2gu5lrczqydnhpr",
            // "terra1erl8suy0fefc94rgt32gyj3qcps243r8w8s3s4duccmexr37pwqq5fmsnq",
            // "terra1p4rhg2ajwvugsnsxaret52v0dpfd7py9545rat806tuw8asmkxjql7c8vj",
            // "terra13mn02luaq4jrw3yewhg3lunqc0ykvmng5q2asygxmce25c83wrdqndpp5l",
            // "terra1vxtl9p5ljzu0fajm8395ph2xjtth03jz57vtc4qztsl6xa3elg5qwue97j",
            // "terra1t4dl4vjcvdrudwalux9k526c26duj0temfph7j2wmc645dm73des7p6mtm",
            // "terra12yr4x4deluqjqfqhugmqkr08yuempkqmduafzsnk68xy2zmls3hq858wq3",
            // "terra1h7u3aaufk6k7fudpky0nrmpfcdslyt3kld48kzcj5yy4h022v3zqyt5lxz",
            // "terra19757sl7n9cup6m55dzwzdtq3dhv252309pw30xxgwqf7gvtuejeszed8q9",
            // "terra18cv2zauwks8g920vmvalwgqcj0nfaj3p5jedvxn8l524w4jaqn2sc5u6ha",
            // "terra1a6vkzkg2d6wx8yrt3pmwwxe76sqdjts0uvv5qnn97ufp6ynwrjas9k2e32",
            // "terra1h0h3v4ytkxwcccvptnp7dusygjaz20r995mxw7947zzv8dql6msqt70ytt",
        ]
    );
});

const createWarpAccount = async(executor: Executor, warp_controller_address: string, uluna_deposit: number): Promise<void> => {
    try {
        await executor.execute(
            warp_controller_address,
            {
                create_account: {}
            },
            {
                coins: [new Coin('uluna', uluna_deposit)],
            }
        )
    } catch (e) {
        console.log(e);
    }
}

const createMigrationStepsOldWarpJobMultiple = async (refs: Refs, network: string, executor: Executor, warp_controller_address: string, submsgs_limit: number | undefined, daos: string[]): Promise<void> => {
    for (const i in daos) {
        console.log("creating a job for DAO:", daos[i]);
        await createMigrationStepsOldWarpJob(refs, network, executor, warp_controller_address, daos[i], submsgs_limit);
    }
}

const createMigrationStepsOldWarpJob = async (refs: Refs, network: string, executor: Executor, warp_controller_address: string, dao_address: string, submsgs_limit: number | undefined): Promise<void> => {
    try {
        const facade_address = refs.getAddress(network, ENTERPRISE_FACADE);

        const facade_query_msg_encoded = Buffer.from(`{"v2_migration_stage":{"contract":"${dao_address}"}}`).toString('base64');

        const perform_migration_step_msg_encoded = Buffer.from(`{\"perform_next_migration_step\":{\"submsgs_limit\":${submsgs_limit}}}`).toString('base64');

        console.log("perform migration step msg encoded:", perform_migration_step_msg_encoded);

        const vars = `[{"query":{"reinitialize":false,"name":"v2MigrationStage","init_fn":{"query":{"wasm":{"smart":{"contract_addr":"${facade_address}","msg":"${facade_query_msg_encoded}"}}},"selector":"$.stage"},"update_fn":null,"kind":"string","encode":false}}]`;

        console.log("vars:", vars);

        const msgs = `[{\"wasm\":{\"execute\":{\"contract_addr\":\"${dao_address}\",\"msg\":\"${perform_migration_step_msg_encoded}\",\"funds\":[]}}}]`;

        console.log("msgs:", msgs);

        await executor.execute(
            warp_controller_address,
            {
                create_job: {
                    name: `Migration for DAO ${dao_address}`,
                    description: "Performs next migration step for a DAO with migration in progress",
                    labels: [],
                    condition: "{\"expr\":{\"string\":{\"left\":{\"ref\":\"$warp.variable.v2MigrationStage\"},\"right\":{\"simple\":\"migration_in_progress\"},\"op\":\"eq\"}}}",
                    msgs: `[{\"wasm\":{\"execute\":{\"contract_addr\":\"${dao_address}\",\"msg\":\"${perform_migration_step_msg_encoded}\",\"funds\":[]}}}]`,
                    vars: vars,
                    recurring: true,
                    requeue_on_evict: false,
                    reward: "20000",
                }
            }
        );
    } catch (e) {
        console.log(e);
    }
}

const executeWarpJob = async (executor: Executor, id: number): Promise<void> => {
    try {
        await executor.execute(
            WARP_CONTROLLER_ADDRESS,
            {
                execute_job: {
                    id: id.toString()
                }
            },
        );
    } catch (e) {
        console.log(e);
    }
}

const createMigrationStepsWarpJob = async (refs: Refs, network: string, executor: Executor, dao_address: string, submsgs_limit: number | undefined): Promise<void> => {
    try {
        // const facade_address = refs.getAddress(network, ENTERPRISE_FACADE);
        const facade_address = "terra1dzgr060p4hlc54ynu4z75fhky6rchr8xaskhslxr50tf0g5gj4gq7q4tva";

        const facade_query_msg_encoded = Buffer.from(`{"v2_migration_stage":{"contract":"${dao_address}"}}`).toString('base64');

        const perform_migration_step_msg_encoded = Buffer.from(`{"perform_next_migration_step":{"submsgs_limit":${submsgs_limit}}`).toString('base64');

        const vars = `[{"query":{"reinitialize":false,"name":"v2MigrationStage","init_fn":{"query":{"wasm":{"smart":{"contract_addr":"${facade_address}","msg":"${facade_query_msg_encoded}"}}},"selector":"$.stage"},"update_fn":null,"kind":"string","encode":false}}]`;

        console.log("vars:", vars);

        await executor.execute(
            "terra1fqcfh8vpqsl7l5yjjtq5wwu6sv989txncq5fa756tv7lywqexraq5vnjvt",
            {
                create_job: {
                    name: "Test migration",
                    description: "Migrates a 'stuck' migration of a DAO",
                    labels: [],
                    executions: [
                        {
                            condition: "{\"expr\":{\"string\":{\"left\":{\"ref\":\"$warp.variable.v2MigrationStage\"},\"right\":{\"simple\":\"migration_in_progress\"},\"op\":\"eq\"}}}",
                            msgs: `[{\"wasm\":{\"execute\":{\"contract_addr\":\"${dao_address}\",\"msg\":\"${perform_migration_step_msg_encoded}\",\"funds\":[]}}}]`,
                        },
                    ],
                    terminate_condition: "{\"expr\":{\"string\":{\"left\":{\"ref\":\"$warp.variable.v2MigrationStage\"},\"right\":{\"simple\":\"migration_completed\"},\"op\":\"eq\"}}}",
                    vars: vars,
                    recurring: true,
                    requeue_on_evict: false,
                    reward: "20000",
                    duration_days: "730",
                }
            }
        );
    } catch (e) {
        console.log(e);
    }
}

const waitForNewBlock = async (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 5000))

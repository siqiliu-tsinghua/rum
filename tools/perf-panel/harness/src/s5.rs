pub struct Workload {
    pub setup: &'static str,
    pub body: &'static str,
    pub item: &'static str,
    pub prompts: &'static [&'static str],
}

pub fn workload(label: &str) -> Option<Workload> {
    Some(match label {
        "s5-down-rhs" => Workload {
            setup: "f[x_]:=(InputString[\"rhs\"];7);",
            body: "f[1]",
            item: "7",
            prompts: &["rhs"],
        },
        "s5-down-guard" => Workload {
            setup: "f[0]=0;f[1]=1;f[2]=2;f[3]=3;f[4]=4;f[5]=5;f[6]=6;f[7]=7;f[8]=8;f[9]=9;f[10]=10;f[11]=11;f[12]=12;f[13]=13;f[14]=14;f[15]=15;f[16]=16;f[17]=17;f[18]=18;f[19]=19;f[20]=20;f[21]=21;f[22]=22;f[23]=23;f[24]=24;f[25]=25;f[26]=26;f[27]=27;f[28]=28;f[29]=29;f[30]=30;f[31]=31;f[x_ /; (InputString[\"guard\"];True)]:=(InputString[\"rhs\"];7);",
            body: "f[100]",
            item: "7",
            prompts: &["guard", "rhs"],
        },
        "s5-sub-rhs" => Workload {
            setup: "k[1][x_]:=(InputString[\"rhs\"];7);",
            body: "k[1][2]",
            item: "7",
            prompts: &["rhs"],
        },
        "s5-sub-guard" => Workload {
            setup: "k[1][x_ /; (InputString[\"guard\"];True)]:=(InputString[\"rhs\"];7);",
            body: "k[1][2]",
            item: "7",
            prompts: &["guard", "rhs"],
        },
        "s5-up-rhs" => Workload {
            setup: "g /: h[g,x_]:=(InputString[\"rhs\"];7);",
            body: "h[g,2]",
            item: "7",
            prompts: &["rhs"],
        },
        "s5-up-guard" => Workload {
            setup: "g /: h[x_ /; (InputString[\"guard\"];True),g]:=(InputString[\"rhs\"];7);",
            body: "h[2,g]",
            item: "7",
            prompts: &["guard", "rhs"],
        },
        "s5-default-rhs" => Workload {
            setup: "Default[d]:=(InputString[\"rhs\"];7);d[x_.]:={x};",
            body: "d[]",
            item: "List[7]",
            prompts: &["rhs"],
        },
        "s5-default-guard" => Workload {
            setup: "Default[d]:=(InputString[\"rhs\"];7) /; (InputString[\"guard\"];True);",
            body: "Default[d]",
            item: "7",
            prompts: &["guard", "rhs"],
        },
        "s5-default-fallback-rhs" => Workload {
            setup: "Default[d]:=(InputString[\"rhs\"];7);",
            body: "Default[d]",
            item: "7",
            prompts: &["rhs"],
        },
        "s5-set" => Workload {
            setup: "",
            body: "v=(InputString[\"rhs\"];7)",
            item: "7",
            prompts: &["rhs"],
        },
        "s5-upset" => Workload {
            setup: "",
            body: "h[g]^=(InputString[\"rhs\"];7)",
            item: "7",
            prompts: &["rhs"],
        },
        "s5-tagset" => Workload {
            setup: "",
            body: "g /: h[g]=(InputString[\"rhs\"];7)",
            item: "7",
            prompts: &["rhs"],
        },
        "s5-rule-rhs" => Workload {
            setup: "",
            body: "(1 -> (InputString[\"rhs\"];7)) === (1 -> 7)",
            item: "True",
            prompts: &["rhs"],
        },
        "s5-rule-lhs" => Workload {
            setup: "",
            body: "(f[x_,(InputString[\"lhs\"];1)] -> 7) === (f[x_,1] -> 7)",
            item: "True",
            prompts: &["lhs"],
        },
        "s5-destructure" => Workload {
            setup: "a={0};b=0;",
            body: "{a[[(InputString[\"lhs\"];1)]],b}={7,8}",
            item: "List[7, 8]",
            prompts: &["lhs"],
        },
        "s5-lhs-twice" => Workload {
            setup: "",
            body: "f[(InputString[\"first\"];1),(InputString[\"later\"];2)]=7",
            item: "7",
            prompts: &["first", "later"],
        },
        "s5-own-rhs" => Workload {
            setup: "v:=(InputString[\"rhs\"];7);",
            body: "v",
            item: "7",
            prompts: &["rhs"],
        },
        "s5-destructure-twice" => Workload {
            setup: "a={0,0};",
            body: "{a[[(InputString[\"first\"];1)]],a[[(InputString[\"later\"];2)]]}={7,8}",
            item: "List[7, 8]",
            prompts: &["first", "later"],
        },
        "s5-rule-lhs-twice" => Workload {
            setup: "",
            body: "(f[x_,(InputString[\"first\"];1),(InputString[\"later\"];2)] -> 7) === (f[x_,1,2] -> 7)",
            item: "True",
            prompts: &["first", "later"],
        },
        _ => return None,
    })
}

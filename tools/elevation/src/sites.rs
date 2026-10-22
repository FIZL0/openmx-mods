//! The four sites M1 builds, with the exact files and window of each (`notes/elevation.md` says how they
//! were chosen). `elevation fetch` downloads these files and `elevation --out STAGE` (build.sh's data
//! step) builds every site whose files are present.

use crate::Request;
use std::path::Path;

/// A USGS 3DEP 1 m tile: its project and file name under `StagedProducts/Elevation/1m/Projects/`.
pub struct Dem {
    pub project: &'static str,
    pub file: &'static str,
}

/// A NAIP quarter quad on Planetary Computer: its path under `naip/v002/`.
pub struct Naip {
    pub path: &'static str,
}

pub struct Site {
    pub id: &'static str,
    pub name: &'static str,
    pub dem: &'static [Dem],
    pub naip: &'static [Naip],
    pub centre: (f64, f64),
    pub cells: (u32, u32),
    /// The MX vs. ATV Unleashed map whose sky, fog and lights the scene takes (M17): its folder under
    /// `Env\` and its name in the game. `notes/elevation.md` says why each.
    pub sky: (&'static str, &'static str),
}

pub const SITES: &[Site] = &[
    Site {
        id: "elev_glamis",
        name: "Glamis dunes",
        dem: &[Dem {
            project: "CA_SaltonSea_EarthMRI_2021_D21",
            file: "USGS_1M_11_x67y365_CA_SaltonSea_EarthMRI_2021_D21.tif",
        }],
        naip: &[Naip {
            path: "ca/2022/ca_060cm_2022/32115/m_3211508_nw_11_060_20220530.tif",
        }],
        centre: (676634.0, 3648229.0),
        cells: (8, 8),
        sky: ("Nat03", "Glamis"),
    },
    Site {
        id: "elev_ocotillo",
        name: "Ocotillo Wells",
        dem: &[Dem {
            project: "CA_SaltonSea_EarthMRI_2021_D21",
            file: "USGS_1M_11_x58y367_CA_SaltonSea_EarthMRI_2021_D21.tif",
        }],
        naip: &[
            Naip {
                path: "ca/2022/ca_060cm_2022/33116/m_3311664_nw_11_060_20220418.tif",
            },
            Naip {
                path: "ca/2022/ca_060cm_2022/33116/m_3311656_sw_11_060_20220505.tif",
            },
        ],
        centre: (584090.0, 3666421.0),
        cells: (8, 8),
        sky: ("Nat09", "Deadman Wash"),
    },
    Site {
        id: "elev_littlesahara",
        name: "Little Sahara",
        dem: &[Dem {
            project: "UT_StatewideCenSouth_2020_A20",
            file: "USGS_1M_12_x38y440_UT_StatewideCenSouth_2020_A20.tif",
        }],
        naip: &[
            Naip {
                path: "ut/2021/ut_060cm_2021/39112/m_3911222_nw_12_060_20211021.tif",
            },
            Naip {
                path: "ut/2021/ut_060cm_2021/39112/m_3911221_ne_12_060_20211021.tif",
            },
        ],
        centre: (381994.0, 4397606.0),
        cells: (8, 8),
        sky: ("MC04", "Glamis South"),
    },
    Site {
        id: "elev_hatfield",
        name: "Hatfield-McCoy",
        dem: &[Dem {
            project: "WV_FEMAR3_Southcentral_2018_D19",
            file: "USGS_1M_17_x42y417_WV_FEMAR3_Southcentral_2018_D19.tif",
        }],
        naip: &[Naip {
            path: "wv/2020/wv_060cm_2020/37081/m_3708118_sw_17_060_20200706.tif",
        }],
        centre: (425114.0, 4166421.0),
        cells: (8, 8),
        sky: ("Nat15", "Smuggler's Notch"),
    },
];

pub const DEM_URL: &str = "https://prd-tnm.s3.amazonaws.com/StagedProducts/Elevation/1m/Projects";
pub const NAIP_URL: &str = "https://naipeuwest.blob.core.windows.net/naip/v002";
pub const NAIP_TOKEN: &str = "https://planetarycomputer.microsoft.com/api/sas/v1/token/naip";

impl Naip {
    pub fn file(&self) -> &'static str {
        self.path.rsplit('/').next().unwrap()
    }
}

/// `YYYY-MM-DD` (UTC) of a file's modification time: when it was fetched.
pub fn date_of(path: &Path) -> Result<String, String> {
    let t = std::fs::metadata(path)
        .and_then(|m| m.modified())
        .map_err(|e| format!("{}: {e}", path.display()))?;
    let secs = t
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs() as i64;
    Ok(civil(secs.div_euclid(86_400)))
}

/// Days since 1970-01-01 to a calendar date (Howard Hinnant's algorithm).
pub fn civil(days: i64) -> String {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

impl Site {
    /// The request for this site with its files under `from` (`dem/` and `naip/`), or the files missing.
    pub fn request(&self, from: &Path) -> Result<Request, Vec<String>> {
        let dem: Vec<_> = self
            .dem
            .iter()
            .map(|d| from.join("dem").join(d.file))
            .collect();
        let naip: Vec<_> = self
            .naip
            .iter()
            .map(|n| from.join("naip").join(n.file()))
            .collect();
        let missing: Vec<String> = dem
            .iter()
            .chain(&naip)
            .filter(|p| !p.is_file())
            .map(|p| p.display().to_string())
            .collect();
        if !missing.is_empty() {
            return Err(missing);
        }
        let tiles = |v: &[Dem]| {
            let p = v[0].project;
            let t: Vec<_> = v
                .iter()
                .map(|d| d.file.split('_').take(4).collect::<Vec<_>>().join("_"))
                .collect();
            format!("project {p}, tile {}", t.join(", "))
        };
        let quads: Vec<_> = self
            .naip
            .iter()
            .map(|n| n.file().trim_end_matches(".tif"))
            .collect();
        let year = self.naip[0].path.split('/').nth(1).unwrap_or("?");
        let date = date_of(&dem[0]).map_err(|e| vec![e])?;
        Ok(Request {
            id: self.id.into(),
            name: self.name.into(),
            dem,
            naip,
            centre: self.centre,
            cells: self.cells,
            texture: 1024,
            surface: 0,
            data: format!(
                "USGS 3DEP 1 m DEM, {}; USDA NAIP {year} 0.6 m, {} (Microsoft Planetary Computer)",
                tiles(self.dem),
                quads.join(" and ")
            ),
            date,
            sky: None,
        })
    }
}

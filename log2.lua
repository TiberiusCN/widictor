» registerLibrary(
  name="mw_interface",
  functions={}
)
getStatus()
» loadString(#text_chunkName(@mwInit.lua))
getStatus()
call(
  id=1,
  nargs=0,
  args={}
)
» registerLibrary(  
  name="mw_interface",  
  functions={
    loadPackage="mw_interface-loadPackage-2",  
    loadPHPLibrary="mw_interface-loadPHPLibrary-2",  
    frameExists="mw_interface-frameExists-2",  
    newChildFrame="mw_interface-newChildFrame-2",  
    getExpandedArgument="mw_interface-getExpandedArgument-2",  
    getAllExpandedArguments="mw_interface-getAllExpandedArguments-2",  
    expandTemplate="mw_interface-expandTemplate-2",  
    callParserFunction="mw_interface-callParserFunction-2",  
    preprocess="mw_interface-preprocess-2",  
    incrementExpensiveFunctionCount="mw_interface-incrementExpensiveFunctionCount-2",  
    isSubsting="mw_interface-isSubsting-2",  
    getFrameTitle="mw_interface-getFrameTitle-2",  
    setTTL="mw_interface-setTTL-2",  
    addWarning="mw_interface-addWarning-2"
  }
)
getStatus()
cleanupChunks(ids={})
»loadString(#text_chunkName(@mw.lua))
getStatus()
call(
  id=4,
  nargs=0,
  args={}
)
cleanupChunks(#ids(5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19))
call(
  id=12,
  nargs=1,
  args={
    [1]={
      allowEnvFuncs=false
    }
  }
)
»loadString(#text_chunkName(@package.lua))
return(
  nvalues=1,
  values={
    [1]=chunks[20]
  }
)
»registerLibrary(
  name="mw_interface",
  functions={
    getNsIndex="mw_interface-getNsIndex-3",
    pagesInCategory="mw_interface-pagesInCategory-3",
    pagesInNamespace="mw_interface-pagesInNamespace-3",
    usersInGroup="mw_interface-usersInGroup-3",
    interwikiMap="mw_interface-interwikiMap-3"
  }
)
getStatus()
cleanupChunks(#ids(5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19))
»loadString(#text_chunkName(@mw.site.lua))
getStatus()
call(
  id=21,
  nargs=0,
  args={}
)
cleanupChunks(#ids(5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 22))
call(
  id=22,
  nargs=1,
  args={
    [1]={
      siteName="triky",
      server="http://localhost",
      scriptPath="",
      stylePath="/skins",
      currentVersion="1.35.1",
      namespaces={
        [-2]={
          id=-2,
          name="Media",
          canonicalName="Media",
          hasSubpages=false,
          hasGenderDistinction=false,
          isCapitalized=true,
          isContent=false,
          isIncludable=true,
          isMovable=false,
          isSubject=true,
          isTalk=false,
          defaultContentModel=nil,
          aliases={},
          subject=-2
        },
        [-1]={
          id=-1,
          name="Specialis",
          canonicalName="Special",
          hasSubpages=false,
          hasGenderDistinction=false,
          isCapitalized=true,
          isContent=false,
          isIncludable=true,
          isMovable=false,
          isSubject=true,
          isTalk=false,
          defaultContentModel=nil,
          aliases={},
          subject=-1
        },
        [0]={
          id=0,
          name="",
          canonicalName="",
          hasSubpages=false,
          hasGenderDistinction=false,
          isCapitalized=true,
          isContent=true,
          isIncludable=true,
          isMovable=true,
          isSubject=true,
          isTalk=false,
          defaultContentModel=nil,
          aliases={},
          subject=0,
          talk=1,
          associated=1,
          displayName="(principale)"
        },
        [1]={
          id=1,
          name="Disputatio",
          canonicalName="Talk",
          hasSubpages=true,
          hasGenderDistinction=false,
          isCapitalized=true,
          isContent=false,
          isIncludable=true,
          isMovable=true,
          isSubject=false,
          isTalk=true,
          defaultContentModel=nil,
          aliases={},
          subject=0,
          talk=1,
          associated=0
        },
        [2]={
          id=2,
          name="Usor",
          canonicalName="User",
          hasSubpages=true,
          hasGenderDistinction=true,
          isCapitalized=true,
          isContent=false,
          isIncludable=true,
          isMovable=true,
          isSubject=true,
          isTalk=false,
          defaultContentModel=nil,
          aliases={},
          subject=2,
          talk=3,
          associated=3
        },
        [3]={
          id=3,
          name="Disputatio Usoris",
          canonicalName="User talk",
          hasSubpages=true,
          hasGenderDistinction=true,
          isCapitalized=true,
          isContent=false,
          isIncludable=true,
          isMovable=true,
          isSubject=false,
          isTalk=true,
          defaultContentModel=nil,
          aliases={},
          subject=2,
          talk=3,
          associated=2
        },
        [4]={
          id=4,
          name="Triky",
          canonicalName="Project",
          hasSubpages=true,
          hasGenderDistinction=false,
          isCapitalized=true,
          isContent=false,
          isIncludable=true,
          isMovable=true,
          isSubject=true,
          isTalk=false,
          defaultContentModel=nil,
          aliases={},
          subject=4,
          talk=5,
          associated=5
        },
        [5]={
          id=5,
          name="Disputatio Triky",
          canonicalName="Project talk",
          hasSubpages=true,
          hasGenderDistinction=false,
          isCapitalized=true,
          isContent=false,
          isIncludable=true,
          isMovable=true,
          isSubject=false,
          isTalk=true,
          defaultContentModel=nil,
          aliases={},
          subject=4,
          talk=5,
          associated=4
        },
        [6]={
          id=6,
          name="Fasciculus",
          canonicalName="File",
          hasSubpages=false,
          hasGenderDistinction=false,
          isCapitalized=true,
          isContent=false,
          isIncludable=true,
          isMovable=true,
          isSubject=true,
          isTalk=false,
          defaultContentModel=nil,
          aliases={
            [1]="Imago",
            [2]="Image"
          },
          subject=6,
          talk=7,
          associated=7
        },
        [7]={
          id=7,
          name="Disputatio Fasciculi",
          canonicalName="File talk",
          hasSubpages=true,
          hasGenderDistinction=false,
          isCapitalized=true,
          isContent=false,
          isIncludable=true,
          isMovable=true,
          isSubject=false,
          isTalk=true,
          defaultContentModel=nil,
          aliases={
            [1]="Disputatio_Imaginis",
            [2]="Image_talk"
          },
          subject=6,
          talk=7,
          associated=6},
        [8]={
          id=8,
          name="MediaWiki",
          canonicalName="MediaWiki",
          hasSubpages=true,
          hasGenderDistinction=false,
          isCapitalized=true,
          isContent=false,
          isIncludable=true,
          isMovable=true,
          isSubject=true,
          isTalk=false,
          defaultContentModel=nil,
          aliases={},
          subject=8,
          talk=9,
          associated=9
        },
        [9]={
          id=9,
          name="Disputatio MediaWiki",
          canonicalName="MediaWiki talk",
          hasSubpages=true,
          hasGenderDistinction=false,
          isCapitalized=true,
          isContent=false,
          isIncludable=true,
          isMovable=true,
          isSubject=false,
          isTalk=true,
          defaultContentModel=nil,
          aliases={},
          subject=8,
          talk=9,
          associated=8
        },
        [10]={
          id=10,
          name="Formula",
          canonicalName="Template",
          hasSubpages=true,
          hasGenderDistinction=false,
          isCapitalized=true,
          isContent=false,
          isIncludable=true,
          isMovable=true,
          isSubject=true,
          isTalk=false,
          defaultContentModel=nil,
          aliases={},
          subject=10,
          talk=11,
          associated=11
        },
        [11]={
          id=11,
          name="Disputatio Formulae",
          canonicalName="Template talk",
          hasSubpages=true,
          hasGenderDistinction=false,
          isCapitalized=true,
          isContent=false,
          isIncludable=true,
          isMovable=true,
          isSubject=false,
          isTalk=true,
          defaultContentModel=nil,
          aliases={},
          subject=10,
          talk=11,
          associated=10
        },
        [12]={
          id=12,
          name="Auxilium",
          canonicalName="Help",
          hasSubpages=true,
          hasGenderDistinction=false,
          isCapitalized=true,
          isContent=false,
          isIncludable=true,
          isMovable=true,
          isSubject=true,
          isTalk=false,
          defaultContentModel=nil,
          aliases={},
          subject=12,
          talk=13,
          associated=13
        },
        [13]={
          id=13,
          name="Disputatio Auxilii",
          canonicalName="Help talk",
          hasSubpages=true,
          hasGenderDistinction=false,
          isCapitalized=true,
          isContent=false,
          isIncludable=true,
          isMovable=true,
          isSubject=false,
          isTalk=true,
          defaultContentModel=nil,
          aliases={},
          subject=12,
          talk=13,
          associated=12
        },
        [14]={
          id=14,
          name="Categoria",
          canonicalName="Category",
          hasSubpages=false,
          hasGenderDistinction=false,
          isCapitalized=true,
          isContent=false,
          isIncludable=true,
          isMovable=true,
          isSubject=true,
          isTalk=false,
          defaultContentModel=nil,
          aliases={},
          subject=14,
          talk=15,
          associated=15
        },
        [15]={
          id=15,
          name="Disputatio Categoriae",
          canonicalName="Category talk",
          hasSubpages=true,
          hasGenderDistinction=false,
          isCapitalized=true,
          isContent=false,
          isIncludable=true,
          isMovable=true,
          isSubject=false,
          isTalk=true,
          defaultContentModel=nil,
          aliases={},
          subject=14,
          talk=15,
          associated=14
        },
        [828]={
          id=828,
          name="Module",
          canonicalName="Module",
          hasSubpages=true,
          hasGenderDistinction=false,
          isCapitalized=true,
          isContent=false,
          isIncludable=true,
          isMovable=true,
          isSubject=true,
          isTalk=false,
          defaultContentModel=nil,
          aliases={},
          subject=828,
          talk=829,
          associated=829
        },
        [829]={
          id=829,
          name="Module talk",
          canonicalName="Module talk",
          hasSubpages=true,
          hasGenderDistinction=false,
          isCapitalized=true,
          isContent=false,
          isIncludable=true,
          isMovable=true,
          isSubject=false,
          isTalk=true,
          defaultContentModel=nil,
          aliases={},
          subject=828,
          talk=829,
          associated=828
        }
      },
      stats={
        pages=3,
        articles=0,
        files=0,
        edits=12,
        users=1,
        activeUsers=0,
        admins=1
      }
    }
  }
)
»registerLibrary(
  name="mw_interface",
  functions={
    anchorEncode="mw_interface-anchorEncode-4",
    localUrl="mw_interface-localUrl-4",
    fullUrl="mw_interface-fullUrl-4",
    canonicalUrl="mw_interface-canonicalUrl-4"
  }
)
getStatus()
cleanupChunks(#ids(5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19))
»loadString(#text_chunkName(@mw.uri.lua))
getStatus()
call(
  id=23,
  nargs=0,
  args={}
)
return(
  nvalues=1,
  values={
    [1]=nil
  }
)
?»loadString(#text_chunkName(@libraryUtil.lua))
return(
  nvalues=1,
  values={
    [1]=chunks[24]
  }
)
cleanupChunks(#ids(5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35))
call(
  id=30,
  nargs=1,
  args={
    [1]={
      defaultUrl="http://localhost/index.php?title=Sampla"
    }
  }
)
»registerLibrary(
  name="mw_interface",
  functions={
    find="mw_interface-find-5",
    match="mw_interface-match-5",
    gmatch_init="mw_interface-gmatch_init-5",
    gmatch_callback="mw_interface-gmatch_callback-5",
    gsub="mw_interface-gsub-5"
  }
)
getStatus()
cleanupChunks(#ids(5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19))
»loadString(#text_chunkName(@mw.ustring.lua))
getStatus()
call(
  id=36,
  nargs=0,
  args={}
)
return(
  nvalues=1,
  values={
    [1]=nil
  }
)
?»loadString(#text_chunkName(@ustring.lua))
return(
  nvalues=1,
  values={
    [1]=chunks[37]
  }
)
cleanupChunks(#ids(5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58))
call(
  id=51,
  nargs=1,
  args={
    [1]={
      stringLengthLimit=2097152,
      patternLengthLimit=10000
    }
  }
)
»registerLibrary(
  name="mw_interface",
  functions={
    getContLangCode="mw_interface-getContLangCode-6",
    isSupportedLanguage="mw_interface-isSupportedLanguage-6",
    isKnownLanguageTag="mw_interface-isKnownLanguageTag-6",
    isValidCode="mw_interface-isValidCode-6",
    isValidBuiltInCode="mw_interface-isValidBuiltInCode-6",
    fetchLanguageName="mw_interface-fetchLanguageName-6",
    fetchLanguageNames="mw_interface-fetchLanguageNames-6",
    getFallbacksFor="mw_interface-getFallbacksFor-6",
    lcfirst="mw_interface-lcfirst-6",
    ucfirst="mw_interface-ucfirst-6",
    lc="mw_interface-lc-6",
    uc="mw_interface-uc-6",
    caseFold="mw_interface-caseFold-6",
    formatNum="mw_interface-formatNum-6",
    formatDate="mw_interface-formatDate-6",
    formatDuration="mw_interface-formatDuration-6",
    getDurationIntervals="mw_interface-getDurationIntervals-6",
    parseFormattedNumber="mw_interface-parseFormattedNumber-6",
    convertPlural="mw_interface-convertPlural-6",
    convertGrammar="mw_interface-convertGrammar-6",
    gender="mw_interface-gender-6",
    isRTL="mw_interface-isRTL-6"
  }
)
getStatus()
cleanupChunks(#ids(5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19))
»loadString(#text_chunkName(@mw.language.lua))
getStatus()
call(
  id=59,
  nargs=0,
  args={
  }
)
cleanupChunks(#ids(5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69))
call(
  id=66,
  nargs=1,
  args={
    [1]={}
  }
)
return(
  nvalues=1,
  values={
    [1]="la"
  }
)
»registerLibrary(
  name="mw_interface",
  functions={
    plain="mw_interface-plain-7",
    check="mw_interface-check-7"
  }
)
getStatus()
cleanupChunks(#ids(5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19))
»loadString(#text_chunkName(@mw.message.lua))
getStatus()
call(
  id=70,
  nargs=0,
  args={}
)
cleanupChunks(#ids(5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 71, 72, 73, 74, 75, 76, 77))
call(
  id=71,
  nargs=1,
  args={
    [1]={
      lang="la"
    }
  }
)
»registerLibrary(
  name="mw_interface",
  functions={
    newTitle="mw_interface-newTitle-8",
    makeTitle="mw_interface-makeTitle-8",
    getExpensiveData="mw_interface-getExpensiveData-8",
    getUrl="mw_interface-getUrl-8",
    getContent="mw_interface-getContent-8",
    getFileInfo="mw_interface-getFileInfo-8",
    protectionLevels="mw_interface-protectionLevels-8",
    cascadingProtection="mw_interface-cascadingProtection-8",
    redirectTarget="mw_interface-redirectTarget-8",
    recordVaryFlag="mw_interface-recordVaryFlag-8"
  }
)
getStatus()
cleanupChunks(#ids(5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19))
»loadString(#text_chunkName(@mw.title.lua))
getStatus()
call(
  id=78,
  nargs=0,
  args={}
)
cleanupChunks(#ids(5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 79, 80, 81, 82, 83))
call(
  id=79,
  nargs=1,
  args={
    [1]={
      thisTitle={
        isCurrentTitle=true,
        isLocal=true,
        interwiki="",
        namespace=0,
        nsText="",
        text="Sampla",
        fragment="",
        thePartialUrl="Sampla",
        file=false
      },
      NS_MEDIA=-2
    }
  }
)
»registerLibrary(
  name="mw_interface",
  functions={
    unstrip="mw_interface-unstrip-9",
    unstripNoWiki="mw_interface-unstripNoWiki-9",
    killMarkers="mw_interface-killMarkers-9",
    getEntityTable="mw_interface-getEntityTable-9",
    jsonEncode="mw_interface-jsonEncode-9",
    jsonDecode="mw_interface-jsonDecode-9"
  }
)
getStatus()
cleanupChunks(#ids(5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19))
»loadString(#text_chunkName(@mw.text.lua))
getStatus()
call(
  id=84,
  nargs=0,
  args={}
)
cleanupChunks(#ids(5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99))
call(
  id=95,
  nargs=1,
  args={
    [1]={
      comma=",",
      and=" et ",
      ellipsis="...",
      nowiki_protocols={
        ([Bb][Ii][Tt][Cc][Oo][Ii][Nn]):="%1&#58;",
        ([Gg][Ee][Oo]):="%1&#58;",
        ([Mm][Aa][Gg][Nn][Ee][Tt]):="%1&#58;",
        ([Mm][Aa][Ii][Ll][Tt][Oo]):="%1&#58;",
        ([Nn][Ee][Ww][Ss]):="%1&#58;",
        ([Ss][Ii][Pp]):="%1&#58;",
        ([Ss][Ii][Pp][Ss]):="%1&#58;",
        ([Ss][Mm][Ss]):="%1&#58;",
        ([Tt][Ee][Ll]):="%1&#58;",
        ([Uu][Rr][Nn]):="%1&#58;",
        ([Xx][Mm][Pp][Pp]):="%1&#58;"
      }
    }
  }
)
»registerLibrary(
  name="mw_interface",
  functions={}
)
getStatus()
cleanupChunks(#ids(5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19))
»loadString(#text_chunkName(@mw.html.lua))
getStatus()
call(
  id=100,
  nargs=0,
  args={}
)
cleanupChunks(#ids(5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 101, 102))
call(
  id=102,
  nargs=1,
  args={
    [1]={
      uniqPrefix="'\"`UNIQ-",
      uniqSuffix="-QINU`\"'"
    }
  }
)
»registerLibrary(
  name="mw_interface",
  functions={
    listAlgorithms="mw_interface-listAlgorithms-11",
    hashValue="mw_interface-hashValue-11"
  }
)
getStatus()
cleanupChunks(#ids(5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19))
»loadString(#text_chunkName(@mw.hash.lua))
getStatus()
call(
  id=103,
  nargs=0,
  args={}
)
cleanupChunks(#ids(5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 104, 105, 106))
call(
  id=106,
  nargs=1,
  args={
    [1]={}
  }
)
getStatus()
cleanupChunks(#ids(5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19))
loadString(#text_chunkName(=Module:Etymology/templates))
getStatus()
call(
  id=19,
  nargs=2,
  args={
    [1]=chunks[107],
    [2]="hello"
  }
)
return(
  nvalues=1,
  values={
    [1]=true
  }
)
getStatus()
getStatus()
call(
  id=18,
  nargs=1,
  args={
    [1]=chunks[108]
  }
)
getStatus()
getStatus()
getStatus()
cleanupChunks(#ids(5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 107))
call(
  id=13,
  nargs=0,
  args={}
)
